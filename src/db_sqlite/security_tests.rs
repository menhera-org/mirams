use super::*;

#[test]
fn password_reset_updates_one_account_and_revokes_its_tokens() {
    let db = SqliteConnection::open_memory().unwrap();
    let store = crate::Store::new(db);
    let users = store.users();
    users.set_password("alice", "old").unwrap();
    users.set_password("bob", "unchanged").unwrap();
    let old_key = users.generate_api_key("alice").unwrap();
    let bob_key = users.generate_api_key("bob").unwrap();
    users.set_password("alice", "new").unwrap();
    assert_eq!(users.list_users().unwrap(), ["alice", "bob"]);
    assert!(!users.check_password("alice", "old").unwrap());
    assert!(users.check_password("alice", "new").unwrap());
    assert!(users.get_user_from_api_key(&old_key).unwrap().is_none());
    assert_eq!(
        users.get_user_from_api_key(&bob_key).unwrap().as_deref(),
        Some("bob")
    );
    let new_key = users.generate_api_key("alice").unwrap();
    assert_ne!(new_key, old_key);
    assert_eq!(
        users.get_user_from_api_key(&new_key).unwrap().as_deref(),
        Some("alice")
    );
}

#[test]
fn migration_repairs_duplicate_users_and_preserves_unaffected_sessions() {
    let pool = r2d2::Pool::new(SqliteConnectionManager::memory()).unwrap();
    let db = SqliteConnection { pool };
    {
        let conn = db.get_conn().unwrap();
        conn.execute_batch(SCHEMA_VERSION_TABLE).unwrap();
        conn.execute_batch(MIGRATION_1).unwrap();
        conn.execute("INSERT INTO schema_version VALUES (1, 1)", [])
            .unwrap();
        for (name, password) in [("alice", "old"), ("alice", "new"), ("bob", "unchanged")] {
            let hash = crate::user::hash_password(password).unwrap();
            conn.execute(
                "INSERT INTO user (name, hashed_password) VALUES (?, ?)",
                [name, &hash],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO api_key (key, user_id) VALUES (?, ?)",
                rusqlite::params![
                    format!("key-{}", conn.last_insert_rowid()),
                    conn.last_insert_rowid()
                ],
            )
            .unwrap();
        }
    }
    db.initialize().unwrap();
    db.initialize().unwrap();
    let store = crate::Store::new(db);
    let users = store.users();
    assert_eq!(users.list_users().unwrap(), ["alice", "bob"]);
    assert!(users.check_password("alice", "new").unwrap());
    assert!(!users.check_password("alice", "old").unwrap());
    assert!(users.get_user_from_api_key("key-1").unwrap().is_none());
    assert!(users.get_user_from_api_key("key-2").unwrap().is_none());
    assert_eq!(
        users.get_user_from_api_key("key-3").unwrap().as_deref(),
        Some("bob")
    );
}

#[test]
fn legacy_prefixes_with_host_bits_still_block_overlapping_allocations() {
    let db = SqliteConnection::open_memory().unwrap();
    let store = crate::Store::new(db.clone());
    crate::example_data::add_example_data(store.clone());
    let conn = db.get_conn().unwrap();
    for (family, pool_prefix, assignment_prefix) in [
        ("ipv4", vec![192, 168, 2, 128], vec![192, 168, 1, 192]),
        (
            "ipv6",
            "2001:db8:2::8000"
                .parse::<std::net::Ipv6Addr>()
                .unwrap()
                .octets()
                .to_vec(),
            "2001:db8:1::c000"
                .parse::<std::net::Ipv6Addr>()
                .unwrap()
                .octets()
                .to_vec(),
        ),
    ] {
        let len = if family == "ipv4" { 24 } else { 112 };
        conn.execute(&format!("INSERT INTO assignment_pool_{family} (name, description, pool_visibility, assignment_space_id, {family}_prefix, {family}_prefix_len) VALUES ('legacy', '', 0, 1, ?, ?)"), rusqlite::params![pool_prefix, len]).unwrap();
        conn.execute(&format!("INSERT INTO assignment_{family} (name, description, assignment_visibility, assignment_pool_id, {family}_prefix, {family}_prefix_len) VALUES ('legacy', '', 0, 1, ?, ?)"), rusqlite::params![assignment_prefix, len + 1]).unwrap();
    }
    drop(conn);
    let v4 = store.ipv4_assignments();
    assert!(v4
        .create_pool(&crate::ipv4::AssignmentPoolIpv4 {
            ipv4_prefix: [192, 168, 2, 0],
            ipv4_prefix_len: 25,
            ..v4.get_pool(1).unwrap()
        })
        .is_err());
    assert!(v4
        .create_assignment(&crate::ipv4::AssignmentIpv4 {
            ipv4_prefix: [192, 168, 1, 128],
            ipv4_prefix_len: 26,
            ..v4.get_assignment(1).unwrap()
        })
        .is_err());
    let v6 = store.ipv6_assignments();
    assert!(v6
        .create_pool(&crate::ipv6::AssignmentPoolIpv6 {
            ipv6_prefix: "2001:db8:2::"
                .parse::<std::net::Ipv6Addr>()
                .unwrap()
                .octets(),
            ipv6_prefix_len: 113,
            ..v6.get_pool(1).unwrap()
        })
        .is_err());
    assert!(v6
        .create_assignment(&crate::ipv6::AssignmentIpv6 {
            ipv6_prefix: "2001:db8:1::8000"
                .parse::<std::net::Ipv6Addr>()
                .unwrap()
                .octets(),
            ipv6_prefix_len: 114,
            ..v6.get_assignment(1).unwrap()
        })
        .is_err());
}
