use super::*;
use crate::assignment::{Assignment, AssignmentRange, ResourceType};

fn legacy_database() -> SqliteConnection {
    let db = SqliteConnection {
        pool: r2d2::Pool::new(SqliteConnectionManager::memory()).unwrap(),
    };
    {
        let conn = db.get_conn().unwrap();
        conn.execute_batch(SCHEMA_VERSION_TABLE).unwrap();
        conn.execute_batch(MIGRATION_1).unwrap();
        conn.execute_batch(MIGRATION_2).unwrap();
        conn.execute("INSERT INTO schema_version VALUES (2, 2)", [])
            .unwrap();
    }
    db
}

fn add_legacy_tree(db: &SqliteConnection, family: &str) {
    let conn = db.get_conn().unwrap();
    for (level, kind, parent_column) in [
        (0, "space", ""),
        (1, "pool", ", assignment_space_id"),
        (2, "assignment", ", assignment_pool_id"),
    ] {
        let table = if level == 2 {
            format!("assignment_{family}")
        } else {
            format!("assignment_{kind}_{family}")
        };
        let parent_value = if level == 0 { "" } else { ", 1" };
        let visibility = if level == 1 { 1 } else { 0 };
        let name = format!("{family} {kind}");
        if family == "asn" {
            let (columns, range) = if level == 2 {
                ("asn", "65000")
            } else if level == 1 {
                ("asn_from, asn_to", "65000, 65099")
            } else {
                ("asn_from, asn_to", "65000, 65199")
            };
            conn.execute(&format!("INSERT INTO {table} (id, name, description, {kind}_visibility, {columns}{parent_column}) VALUES (1, ?, 'preserved description', ?, {range}{parent_value})"), rusqlite::params![name, visibility]).unwrap();
        } else {
            let (prefix, len): (Vec<u8>, i32) = match (family, level) {
                ("ipv4", 0) => (vec![192, 168, 0, 99], 16),
                ("ipv4", 1) => (vec![192, 168, 1, 99], 24),
                ("ipv4", _) => (vec![192, 168, 1, 1], 32),
                ("ipv6", 0) => (
                    "2001:db8::99"
                        .parse::<std::net::Ipv6Addr>()
                        .unwrap()
                        .octets()
                        .to_vec(),
                    32,
                ),
                ("ipv6", 1) => (
                    "2001:db8:1::99"
                        .parse::<std::net::Ipv6Addr>()
                        .unwrap()
                        .octets()
                        .to_vec(),
                    48,
                ),
                _ => (
                    "2001:db8:1:1::99"
                        .parse::<std::net::Ipv6Addr>()
                        .unwrap()
                        .octets()
                        .to_vec(),
                    64,
                ),
            };
            conn.execute(&format!("INSERT INTO {table} (id, name, description, {kind}_visibility, {family}_prefix, {family}_prefix_len{parent_column}) VALUES (1, ?, 'preserved description', ?, ?, ?{parent_value})"), rusqlite::params![name, visibility, prefix, len]).unwrap();
        }
    }
}

#[test]
fn migrates_all_legacy_types_with_colliding_ids_and_normalizes_prefixes() {
    let db = legacy_database();
    for family in ["asn", "ipv4", "ipv6"] {
        add_legacy_tree(&db, family);
    }
    db.initialize().unwrap();
    let store = crate::Store::new(db.clone());
    let assignments = store.assignments();
    let roots = assignments.get_assignments(None, None).unwrap();
    assert_eq!(roots.len(), 3);
    let mut ids = std::collections::HashSet::new();
    for root in roots {
        assert!(ids.insert(root.id));
        assert_eq!(root.parent_id, None);
        let family = root.range.resource_type().as_str();
        assert_eq!(root.name, format!("{family} space"));
        assert_eq!(root.description, "preserved description");
        assert_eq!(root.visibility, ObjectVisibility::Public);
        let children = assignments.get_assignments(Some(root.id), None).unwrap();
        assert_eq!(children.len(), 1);
        let child = &children[0];
        assert!(ids.insert(child.id));
        assert_eq!(child.name, format!("{family} pool"));
        assert_eq!(child.visibility, ObjectVisibility::Private);
        assert!(root.range.contains(&child.range).unwrap());
        let leaves = assignments.get_assignments(Some(child.id), None).unwrap();
        assert_eq!(leaves.len(), 1);
        let leaf = &leaves[0];
        assert!(ids.insert(leaf.id));
        assert_eq!(leaf.name, format!("{family} assignment"));
        assert_eq!(leaf.visibility, ObjectVisibility::Public);
        assert!(!assignments.is_public(leaf.id).unwrap());
        assert_eq!(
            assignments.get_ancestors(leaf.id).unwrap(),
            vec![root.clone(), child.clone()]
        );
        match leaf.range {
            AssignmentRange::Asn { asn_from, asn_to } => {
                assert_eq!((asn_from, asn_to), (65000, 65000))
            }
            AssignmentRange::Ipv4 {
                ipv4_prefix,
                ipv4_prefix_len,
            } => assert_eq!((ipv4_prefix, ipv4_prefix_len), ([192, 168, 1, 1], 32)),
            AssignmentRange::Ipv6 {
                ipv6_prefix,
                ipv6_prefix_len,
            } => {
                assert_eq!(ipv6_prefix_len, 64);
                assert_eq!(&ipv6_prefix[8..], &[0; 8]);
            }
        }
    }
    assert_eq!(ids.len(), 9);
    db.initialize().unwrap();
    assert_eq!(assignments.get_assignments(None, None).unwrap().len(), 3);
    let conn = db.get_conn().unwrap();
    let old_tables: i32 = conn.query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND (name LIKE 'assignment_space_%' OR name LIKE 'assignment_pool_%' OR name IN ('assignment_asn', 'assignment_ipv4', 'assignment_ipv6'))", [], |row| row.get(0)).unwrap();
    assert_eq!(old_tables, 0);
}

#[test]
fn invalid_legacy_hierarchy_rolls_back_the_entire_migration() {
    let db = legacy_database();
    add_legacy_tree(&db, "asn");
    db.get_conn()
        .unwrap()
        .execute("UPDATE assignment_pool_asn SET asn_from=64000", [])
        .unwrap();
    let error = db.initialize().unwrap_err();
    assert!(error.to_string().contains("outside its parent"));
    let conn = db.get_conn().unwrap();
    assert_eq!(
        conn.query_row("SELECT MAX(version) FROM schema_version", [], |row| row
            .get::<_, i32>(0))
            .unwrap(),
        2
    );
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM assignment_pool_asn", [], |row| row
            .get::<_, i32>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE name='assignments'",
            [],
            |row| row.get::<_, i32>(0)
        )
        .unwrap(),
        0
    );
}

#[test]
fn first_file_open_migrates_and_reopening_keeps_ids_and_sessions() {
    let path = std::env::temp_dir().join(format!(
        "mirams-migration-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch(SCHEMA_VERSION_TABLE).unwrap();
        conn.execute_batch(MIGRATION_1).unwrap();
        conn.execute_batch(MIGRATION_2).unwrap();
        conn.execute("INSERT INTO schema_version VALUES (2, 2)", [])
            .unwrap();
        conn.execute(
            "INSERT INTO user (id,name,hashed_password) VALUES (1,'alice','unchanged')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO api_key (key,user_id) VALUES ('existing-token',1)",
            [],
        )
        .unwrap();
        for name in ["one", "overlapping root"] {
            conn.execute("INSERT INTO assignment_space_asn (name,description,space_visibility,asn_from,asn_to) VALUES (?,'',0,65000,65535)",[name]).unwrap();
        }
    }
    let ids = {
        let db = SqliteConnection::open_file(path.to_str().unwrap()).unwrap();
        let store = crate::Store::new(db);
        assert_eq!(
            store
                .users()
                .get_user_from_api_key("existing-token")
                .unwrap()
                .as_deref(),
            Some("alice")
        );
        store
            .assignments()
            .get_assignments(None, Some(ResourceType::Asn))
            .unwrap()
            .iter()
            .map(|a| a.id)
            .collect::<Vec<_>>()
    };
    assert_eq!(ids.len(), 2);
    {
        let db = SqliteConnection::open_file(path.to_str().unwrap()).unwrap();
        let store = crate::Store::new(db);
        assert_eq!(
            store
                .assignments()
                .get_assignments(None, Some(ResourceType::Asn))
                .unwrap()
                .iter()
                .map(|a| a.id)
                .collect::<Vec<_>>(),
            ids
        );
        let parent = store.assignments().get_assignment(ids[0]).unwrap();
        let child = Assignment {
            parent_id: Some(parent.id),
            ..parent
        };
        assert!(store.assignments().create_assignment(&child).is_ok());
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn migration_preserves_historical_sibling_overlaps_without_allowing_new_ones() {
    let db = legacy_database();
    add_legacy_tree(&db, "asn");
    db.get_conn().unwrap().execute("INSERT INTO assignment_pool_asn (name,description,pool_visibility,assignment_space_id,asn_from,asn_to) SELECT 'historical duplicate',description,pool_visibility,assignment_space_id,asn_from,asn_to FROM assignment_pool_asn", []).unwrap();
    db.initialize().unwrap();
    let store = crate::Store::new(db);
    let assignments = store.assignments();
    let root = assignments
        .get_assignments(None, Some(ResourceType::Asn))
        .unwrap()
        .remove(0);
    let children = assignments.get_assignments(Some(root.id), None).unwrap();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].range, children[1].range);
    assert!(assignments.create_assignment(&children[0]).is_err());
}
