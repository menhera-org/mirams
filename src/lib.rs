
pub mod store;
pub mod db_sqlite;

pub mod static_files;
pub mod server;

pub mod types;
pub mod ipv6;
pub mod ipv4;
pub mod asn;
pub mod user;

pub use store::Store;
pub use types::Error;
pub use types::ErrorKind;


pub mod example_data {
    use crate::*;

    pub fn add_example_data<T: store::DbConnection + Clone + Send + Sync + 'static>(store: store::Store<T>) {
        let asn_store = store.asn_assignments();
        let ipv4_store = store.ipv4_assignments();
        let ipv6_store = store.ipv6_assignments();

        let space = asn::AssignmentSpaceAsn {
            id: 0,
            name: "Example ASN space".to_string(),
            description: "Example ASN space".to_string(),
            space_visibility: types::ObjectVisibility::Public,
            asn_from: 65000,
            asn_to: 65199,
        };
        let space_id = asn_store.create_space(&space).unwrap();
        let pool = asn::AssignmentPoolAsn {
            id: 0,
            assignment_space_id: space_id,
            name: "Example ASN pool".to_string(),
            description: "Example ASN pool".to_string(),
            pool_visibility: types::ObjectVisibility::Public,
            asn_from: 65000,
            asn_to: 65099,
        };
        let pool_id = asn_store.create_pool(&pool).unwrap();
        let assignment = asn::AssignmentAsn {
            id: 0,
            assignment_pool_id: pool_id,
            name: "Example ASN assignment".to_string(),
            description: "Example ASN assignment".to_string(),
            assignment_visibility: types::ObjectVisibility::Public,
            asn: 65000,
        };
        asn_store.create_assignment(&assignment).unwrap();

        let space = ipv4::AssignmentSpaceIpv4 {
            id: 0,
            name: "Example IPv4 space".to_string(),
            description: "Example IPv4 space".to_string(),
            space_visibility: types::ObjectVisibility::Public,
            ipv4_prefix: [192, 168, 0, 0],
            ipv4_prefix_len: 16,
        };
        let space_id = ipv4_store.create_space(&space).unwrap();
        let pool = ipv4::AssignmentPoolIpv4 {
            id: 0,
            assignment_space_id: space_id,
            name: "Example IPv4 pool".to_string(),
            description: "Example IPv4 pool".to_string(),
            pool_visibility: types::ObjectVisibility::Public,
            ipv4_prefix: [192, 168, 1, 0],
            ipv4_prefix_len: 24,
        };
        let pool_id = ipv4_store.create_pool(&pool).unwrap();
        let assignment = ipv4::AssignmentIpv4 {
            id: 0,
            assignment_pool_id: pool_id,
            name: "Example IPv4 assignment".to_string(),
            description: "Example IPv4 assignment".to_string(),
            assignment_visibility: types::ObjectVisibility::Public,
            ipv4_prefix: [192, 168, 1, 1],
            ipv4_prefix_len: 32,
        };
        ipv4_store.create_assignment(&assignment).unwrap();

        let space = ipv6::AssignmentSpaceIpv6 {
            id: 0,
            name: "Example IPv6 space".to_string(),
            description: "Example IPv6 space".to_string(),
            space_visibility: types::ObjectVisibility::Public,
            ipv6_prefix: "2001:db8::".parse::<std::net::Ipv6Addr>().unwrap().octets(),
            ipv6_prefix_len: 32,
        };
        let space_id = ipv6_store.create_space(&space).unwrap();
        let pool = ipv6::AssignmentPoolIpv6 {
            id: 0,
            assignment_space_id: space_id,
            name: "Example IPv6 pool".to_string(),
            description: "Example IPv6 pool".to_string(),
            pool_visibility: types::ObjectVisibility::Public,
            ipv6_prefix: "2001:db8:1::".parse::<std::net::Ipv6Addr>().unwrap().octets(),
            ipv6_prefix_len: 48,
        };
        let pool_id = ipv6_store.create_pool(&pool).unwrap();
        let assignment = ipv6::AssignmentIpv6 {
            id: 0,
            assignment_pool_id: pool_id,
            name: "Example IPv6 assignment".to_string(),
            description: "Example IPv6 assignment".to_string(),
            assignment_visibility: types::ObjectVisibility::Public,
            ipv6_prefix: "2001:db8:1:1::".parse::<std::net::Ipv6Addr>().unwrap().octets(),
            ipv6_prefix_len: 64,
        };
        ipv6_store.create_assignment(&assignment).unwrap();
    }
}


/// Top-level tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_store() {
        let db = db_sqlite::SqliteConnection::open_memory().unwrap();
        let store = Store::new(db);
        let user_store = store.users();
        user_store.set_password("alice", "password").unwrap();
        assert!(user_store.check_password("alice", "password").unwrap());
        assert!(!user_store.check_password("alice", "wrong").unwrap());
        let key = user_store.generate_api_key("alice").unwrap();
        assert_eq!(user_store.get_user_from_api_key(&key).unwrap(), Some("alice".to_string()));
    }

    #[test]
    fn ipv4_masks() {
        use std::net::Ipv4Addr;

        let mask = ipv4::ipv4_subnet_mask(20);
        assert_eq!(mask, Ipv4Addr::new(255, 255, 240, 0).octets());
        let mask = ipv4::ipv4_subnet_mask(16);
        assert_eq!(mask, Ipv4Addr::new(255, 255, 0, 0).octets());
        let mask = ipv4::ipv4_subnet_mask(8);
        assert_eq!(mask, Ipv4Addr::new(255, 0, 0, 0).octets());
        let mask = ipv4::ipv4_subnet_mask(32);
        assert_eq!(mask, Ipv4Addr::new(255, 255, 255, 255).octets());
        let mask = ipv4::ipv4_wildcard_mask(20);
        assert_eq!(mask, Ipv4Addr::new(0, 0, 15, 255).octets());
        let mask = ipv4::ipv4_wildcard_mask(16);
        assert_eq!(mask, Ipv4Addr::new(0, 0, 255, 255).octets());
        let mask = ipv4::ipv4_wildcard_mask(8);
        assert_eq!(mask, Ipv4Addr::new(0, 255, 255, 255).octets());
        let mask = ipv4::ipv4_wildcard_mask(32);
        assert_eq!(mask, Ipv4Addr::new(0, 0, 0, 0).octets());

        let network = ipv4::ipv4_network_address(Ipv4Addr::new(192, 168, 1, 1).octets(), 24);
        assert_eq!(network, Ipv4Addr::new(192, 168, 1, 0).octets());
        let network = ipv4::ipv4_network_address(Ipv4Addr::new(192, 168, 1, 1).octets(), 16);
        assert_eq!(network, Ipv4Addr::new(192, 168, 0, 0).octets());
        let network = ipv4::ipv4_network_address(Ipv4Addr::new(192, 168, 1, 1).octets(), 8);
        assert_eq!(network, Ipv4Addr::new(192, 0, 0, 0).octets());
        let network = ipv4::ipv4_network_address(Ipv4Addr::new(192, 168, 1, 1).octets(), 32);
        assert_eq!(network, Ipv4Addr::new(192, 168, 1, 1).octets());

        let broadcast = ipv4::ipv4_broadcast_address(Ipv4Addr::new(192, 168, 1, 1).octets(), 24);
        assert_eq!(broadcast, Ipv4Addr::new(192, 168, 1, 255).octets());
        let broadcast = ipv4::ipv4_broadcast_address(Ipv4Addr::new(192, 168, 1, 1).octets(), 16);
        assert_eq!(broadcast, Ipv4Addr::new(192, 168, 255, 255).octets());
        let broadcast = ipv4::ipv4_broadcast_address(Ipv4Addr::new(192, 168, 1, 1).octets(), 8);
        assert_eq!(broadcast, Ipv4Addr::new(192, 255, 255, 255).octets());
        let broadcast = ipv4::ipv4_broadcast_address(Ipv4Addr::new(192, 168, 1, 1).octets(), 32);
        assert_eq!(broadcast, Ipv4Addr::new(192, 168, 1, 1).octets());
    }

    #[test]
    fn ipv4_assignment_store() {
        use std::net::Ipv4Addr;

        let db = db_sqlite::SqliteConnection::open_memory().unwrap();
        let store = Store::new(db);
        let ipv4_store = store.ipv4_assignments();
        let space = ipv4::AssignmentSpaceIpv4 {
            id: 0,
            name: "Test assignment space".to_string(),
            description: "Description".to_string(),
            space_visibility: types::ObjectVisibility::Public,
            ipv4_prefix: Ipv4Addr::new(192, 168, 0, 0).octets(),
            ipv4_prefix_len: 16,
        };
        let space_id = ipv4_store.create_space(&space).unwrap();
        let space2 = ipv4::AssignmentSpaceIpv4 {
            id: 0,
            name: "Test assignment space 2".to_string(),
            description: "Description".to_string(),
            space_visibility: types::ObjectVisibility::Public,
            ipv4_prefix: Ipv4Addr::new(172, 16, 0, 0).octets(),
            ipv4_prefix_len: 16,
        };
        ipv4_store.create_space(&space2).unwrap();
        let pool = ipv4::AssignmentPoolIpv4 {
            id: 0,
            assignment_space_id: space_id,
            name: "Test pool".to_string(),
            description: "Description".to_string(),
            pool_visibility: types::ObjectVisibility::Public,
            ipv4_prefix: Ipv4Addr::new(192, 168, 0, 0).octets(),
            ipv4_prefix_len: 24,
        };
        let pool_id = ipv4_store.create_pool(&pool).unwrap();
        let pool2 = ipv4::AssignmentPoolIpv4 {
            id: 0,
            assignment_space_id: space_id,
            name: "Test pool 2".to_string(),
            description: "Description".to_string(),
            pool_visibility: types::ObjectVisibility::Public,
            ipv4_prefix: Ipv4Addr::new(192, 168, 1, 0).octets(),
            ipv4_prefix_len: 24,
        };
        ipv4_store.create_pool(&pool2).unwrap();
        let overlapping_pool_1 = ipv4::AssignmentPoolIpv4 {
            id: 0,
            assignment_space_id: space_id,
            name: "overlapping pool 1".to_string(),
            description: "Description".to_string(),
            pool_visibility: types::ObjectVisibility::Public,
            ipv4_prefix: Ipv4Addr::new(192, 168, 0, 0).octets(),
            ipv4_prefix_len: 24,
        };
        assert!(ipv4_store.create_pool(&overlapping_pool_1).is_err());
        let overlapping_pool_2 = ipv4::AssignmentPoolIpv4 {
            id: 0,
            assignment_space_id: space_id,
            name: "overlapping pool 2".to_string(),
            description: "Description".to_string(),
            pool_visibility: types::ObjectVisibility::Public,
            ipv4_prefix: Ipv4Addr::new(192, 168, 0, 128).octets(),
            ipv4_prefix_len: 25,
        };
        assert!(ipv4_store.create_pool(&overlapping_pool_2).is_err());
        let overlapping_pool_3 = ipv4::AssignmentPoolIpv4 {
            id: 0,
            assignment_space_id: space_id,
            name: "overlapping pool 3".to_string(),
            description: "Description".to_string(),
            pool_visibility: types::ObjectVisibility::Public,
            ipv4_prefix: Ipv4Addr::new(192, 168, 0, 0).octets(),
            ipv4_prefix_len: 23,
        };
        assert!(ipv4_store.create_pool(&overlapping_pool_3).is_err());
        let assignment = ipv4::AssignmentIpv4 {
            id: 0,
            assignment_pool_id: pool_id,
            name: "Test assignment".to_string(),
            description: "Description".to_string(),
            assignment_visibility: types::ObjectVisibility::Public,
            ipv4_prefix: Ipv4Addr::new(192, 168, 0, 1).octets(),
            ipv4_prefix_len: 32,
        };
        let assignment_id = ipv4_store.create_assignment(&assignment).unwrap();
        assert!(ipv4_store.get_space(space_id).is_ok());
        assert!(ipv4_store.get_pool(pool_id).is_ok());
        assert!(ipv4_store.get_assignment(assignment_id).is_ok());
        assert_eq!(ipv4_store.get_pools(space_id).unwrap().len(), 2);
        ipv4_store.delete_space(space_id).unwrap();
        assert!(ipv4_store.get_space(space_id).is_err());
    }

    #[test]
    fn ipv6_masks() {
        use std::net::Ipv6Addr;

        let mask = ipv6::ipv6_subnet_mask(56);
        let assumed_mask = "ffff:ffff:ffff:ff00::".parse::<Ipv6Addr>().unwrap().octets();
        assert_eq!(mask, assumed_mask);

        let mask = ipv6::ipv6_wildcard_mask(80);
        let assumed_mask = "::ffff:ffff:ffff".parse::<Ipv6Addr>().unwrap().octets();
        assert_eq!(mask, assumed_mask);

        let network = ipv6::ipv6_network_address("2001:db8::1".parse::<Ipv6Addr>().unwrap().octets(), 64);
        let assumed_network = "2001:db8::".parse::<Ipv6Addr>().unwrap().octets();
        assert_eq!(network, assumed_network);

        let broadcast = ipv6::ipv6_broadcast_address("2001:db8::1".parse::<Ipv6Addr>().unwrap().octets(), 64);
        let assumed_broadcast = "2001:db8::ffff:ffff:ffff:ffff".parse::<Ipv6Addr>().unwrap().octets();
        assert_eq!(broadcast, assumed_broadcast);
    }

    #[test]
    fn ipv6_assignment_store() {
        use std::net::Ipv6Addr;

        let db = db_sqlite::SqliteConnection::open_memory().unwrap();
        let store = Store::new(db);
        let ipv6_store = store.ipv6_assignments();
        let space = ipv6::AssignmentSpaceIpv6 {
            id: 0,
            name: "Test assignment space".to_string(),
            description: "Description".to_string(),
            space_visibility: types::ObjectVisibility::Public,
            ipv6_prefix: "2001:db8::".parse::<Ipv6Addr>().unwrap().octets(),
            ipv6_prefix_len: 32,
        };
        let space_id = ipv6_store.create_space(&space).unwrap();
        let space2 = ipv6::AssignmentSpaceIpv6 {
            id: 0,
            name: "Test assignment space 2".to_string(),
            description: "Description".to_string(),
            space_visibility: types::ObjectVisibility::Public,
            ipv6_prefix: "fd12:3456::".parse::<Ipv6Addr>().unwrap().octets(),
            ipv6_prefix_len: 32,
        };
        ipv6_store.create_space(&space2).unwrap();
        let pool = ipv6::AssignmentPoolIpv6 {
            id: 0,
            assignment_space_id: space_id,
            name: "Test pool".to_string(),
            description: "Description".to_string(),
            pool_visibility: types::ObjectVisibility::Public,
            ipv6_prefix: "2001:db8::".parse::<Ipv6Addr>().unwrap().octets(),
            ipv6_prefix_len: 48,
        };
        let pool_id = ipv6_store.create_pool(&pool).unwrap();
        let pool2 = ipv6::AssignmentPoolIpv6 {
            id: 0,
            assignment_space_id: space_id,
            name: "Test pool 2".to_string(),
            description: "Description".to_string(),
            pool_visibility: types::ObjectVisibility::Public,
            ipv6_prefix: "2001:db8:1::".parse::<Ipv6Addr>().unwrap().octets(),
            ipv6_prefix_len: 48,
        };
        ipv6_store.create_pool(&pool2).unwrap();
        let overlapping_pool_1 = ipv6::AssignmentPoolIpv6 {
            id: 0,
            assignment_space_id: space_id,
            name: "overlapping pool 1".to_string(),
            description: "Description".to_string(),
            pool_visibility: types::ObjectVisibility::Public,
            ipv6_prefix: "2001:db8::".parse::<Ipv6Addr>().unwrap().octets(),
            ipv6_prefix_len: 48,
        };
        assert!(ipv6_store.create_pool(&overlapping_pool_1).is_err());
        let overlapping_pool_2 = ipv6::AssignmentPoolIpv6 {
            id: 0,
            assignment_space_id: space_id,
            name: "overlapping pool 2".to_string(),
            description: "Description".to_string(),
            pool_visibility: types::ObjectVisibility::Public,
            ipv6_prefix: "2001:db8:1:8000::".parse::<Ipv6Addr>().unwrap().octets(),
            ipv6_prefix_len: 49,
        };
        assert!(ipv6_store.create_pool(&overlapping_pool_2).is_err());
        let overlapping_pool_3 = ipv6::AssignmentPoolIpv6 {
            id: 0,
            assignment_space_id: space_id,
            name: "overlapping pool 3".to_string(),
            description: "Description".to_string(),
            pool_visibility: types::ObjectVisibility::Public,
            ipv6_prefix: "2001:db8::".parse::<Ipv6Addr>().unwrap().octets(),
            ipv6_prefix_len: 47,
        };
        assert!(ipv6_store.create_pool(&overlapping_pool_3).is_err());
        let assignment = ipv6::AssignmentIpv6 {
            id: 0,
            assignment_pool_id: pool_id,
            name: "Test assignment".to_string(),
            description: "Description".to_string(),
            assignment_visibility: types::ObjectVisibility::Public,
            ipv6_prefix: "2001:db8:0:1::".parse::<Ipv6Addr>().unwrap().octets(),
            ipv6_prefix_len: 64,
        };
        let assignment_id = ipv6_store.create_assignment(&assignment).unwrap();
        assert!(ipv6_store.get_space(space_id).is_ok());
        assert!(ipv6_store.get_pool(pool_id).is_ok());
        assert!(ipv6_store.get_assignment(assignment_id).is_ok());
        assert_eq!(ipv6_store.get_pools(space_id).unwrap().len(), 2);
        ipv6_store.delete_space(space_id).unwrap();
        assert!(ipv6_store.get_space(space_id).is_err());
        assert!(ipv6_store.get_assignment(assignment_id).is_err());
    }

    #[test]
    fn asn_store() {
        let db = db_sqlite::SqliteConnection::open_memory().unwrap();
        let store = Store::new(db);
        let asn_store = store.asn_assignments();
        let space = asn::AssignmentSpaceAsn {
            id: 0,
            name: "Test assignment space".to_string(),
            description: "Description".to_string(),
            space_visibility: types::ObjectVisibility::Public,
            asn_from: 65000,
            asn_to: 65199,
        };
        let space_id = asn_store.create_space(&space).unwrap();
        let space2 = asn::AssignmentSpaceAsn {
            id: 0,
            name: "Test assignment space 2".to_string(),
            description: "Description".to_string(),
            space_visibility: types::ObjectVisibility::Public,
            asn_from: 65200,
            asn_to: 65399,
        };
        asn_store.create_space(&space2).unwrap();
        let pool = asn::AssignmentPoolAsn {
            id: 0,
            assignment_space_id: space_id,
            name: "Test pool".to_string(),
            description: "Description".to_string(),
            pool_visibility: types::ObjectVisibility::Public,
            asn_from: 65000,
            asn_to: 65099,
        };
        let pool_id = asn_store.create_pool(&pool).unwrap();
        let pool2 = asn::AssignmentPoolAsn {
            id: 0,
            assignment_space_id: space_id,
            name: "Test pool 2".to_string(),
            description: "Description".to_string(),
            pool_visibility: types::ObjectVisibility::Public,
            asn_from: 65100,
            asn_to: 65199,
        };
        asn_store.create_pool(&pool2).unwrap();
        let overlapping_pool_1 = asn::AssignmentPoolAsn {
            id: 0,
            assignment_space_id: space_id,
            name: "overlapping pool 1".to_string(),
            description: "Description".to_string(),
            pool_visibility: types::ObjectVisibility::Public,
            asn_from: 65000,
            asn_to: 65099,
        };
        assert!(asn_store.create_pool(&overlapping_pool_1).is_err());
        let overlapping_pool_2 = asn::AssignmentPoolAsn {
            id: 0,
            assignment_space_id: space_id,
            name: "overlapping pool 2".to_string(),
            description: "Description".to_string(),
            pool_visibility: types::ObjectVisibility::Public,
            asn_from: 65050,
            asn_to: 65149,
        };
        assert!(asn_store.create_pool(&overlapping_pool_2).is_err());
        let overlapping_pool_3 = asn::AssignmentPoolAsn {
            id: 0,
            assignment_space_id: space_id,
            name: "overlapping pool 3".to_string(),
            description: "Description".to_string(),
            pool_visibility: types::ObjectVisibility::Public,
            asn_from: 65000,
            asn_to: 65098,
        };
        assert!(asn_store.create_pool(&overlapping_pool_3).is_err());
        let assignment = asn::AssignmentAsn {
            id: 0,
            assignment_pool_id: pool_id,
            name: "Test assignment".to_string(),
            description: "Description".to_string(),
            assignment_visibility: types::ObjectVisibility::Public,
            asn: 65000,
        };
        let assignment_id = asn_store.create_assignment(&assignment).unwrap();
        assert!(asn_store.get_space(space_id).is_ok());
        assert!(asn_store.get_pool(pool_id).is_ok());
        assert!(asn_store.get_assignment(assignment_id).is_ok());
        assert_eq!(asn_store.get_pools(space_id).unwrap().len(), 2);
        asn_store.delete_space(space_id).unwrap();
        assert!(asn_store.get_space(space_id).is_err());
        assert!(asn_store.get_assignment(assignment_id).is_err());
    }

    #[test]
    fn static_files() {
        let files = static_files::frontend_files();
        assert!(files.len() > 0);
        for (path, _) in files {
            assert!(path.len() > 0);
            println!("{}", path);
        }
    }
}
