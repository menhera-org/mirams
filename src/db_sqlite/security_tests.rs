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
