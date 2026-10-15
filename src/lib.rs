pub mod db_sqlite;
pub mod store;

pub mod server;
pub mod static_files;

pub mod assignment;
pub mod ipv4;
pub mod ipv6;
pub mod types;
pub mod user;

pub use store::Store;
pub use types::Error;
pub use types::ErrorKind;

pub mod example_data {
    use crate::{
        assignment::{Assignment, AssignmentRange},
        store::{DbConnection, Store},
        types::ObjectVisibility,
    };

    pub fn add_example_data<T: DbConnection + Clone + Send + Sync + 'static>(store: Store<T>) {
        let assignments = store.assignments();
        for (label, ranges) in [
            (
                "ASN",
                [
                    AssignmentRange::Asn {
                        asn_from: 65000,
                        asn_to: 65199,
                    },
                    AssignmentRange::Asn {
                        asn_from: 65000,
                        asn_to: 65099,
                    },
                    AssignmentRange::Asn {
                        asn_from: 65000,
                        asn_to: 65000,
                    },
                ],
            ),
            (
                "IPv4",
                [
                    AssignmentRange::Ipv4 {
                        ipv4_prefix: [192, 168, 0, 0],
                        ipv4_prefix_len: 16,
                    },
                    AssignmentRange::Ipv4 {
                        ipv4_prefix: [192, 168, 1, 0],
                        ipv4_prefix_len: 24,
                    },
                    AssignmentRange::Ipv4 {
                        ipv4_prefix: [192, 168, 1, 1],
                        ipv4_prefix_len: 32,
                    },
                ],
            ),
            (
                "IPv6",
                [
                    AssignmentRange::Ipv6 {
                        ipv6_prefix: "2001:db8::".parse::<std::net::Ipv6Addr>().unwrap().octets(),
                        ipv6_prefix_len: 32,
                    },
                    AssignmentRange::Ipv6 {
                        ipv6_prefix: "2001:db8:1::"
                            .parse::<std::net::Ipv6Addr>()
                            .unwrap()
                            .octets(),
                        ipv6_prefix_len: 48,
                    },
                    AssignmentRange::Ipv6 {
                        ipv6_prefix: "2001:db8:1:1::"
                            .parse::<std::net::Ipv6Addr>()
                            .unwrap()
                            .octets(),
                        ipv6_prefix_len: 64,
                    },
                ],
            ),
        ] {
            let mut parent_id = None;
            for (level, range) in ranges.into_iter().enumerate() {
                parent_id = Some(
                    assignments
                        .create_assignment(&Assignment {
                            id: 0,
                            parent_id,
                            name: format!("Example {label} assignment {}", level + 1),
                            description: String::new(),
                            visibility: ObjectVisibility::Public,
                            range,
                        })
                        .unwrap(),
                );
            }
        }
    }
}

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
        assert_eq!(
            user_store.get_user_from_api_key(&key).unwrap(),
            Some("alice".to_string())
        );
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
    fn ipv6_masks() {
        use std::net::Ipv6Addr;

        let mask = ipv6::ipv6_subnet_mask(56);
        let assumed_mask = "ffff:ffff:ffff:ff00::"
            .parse::<Ipv6Addr>()
            .unwrap()
            .octets();
        assert_eq!(mask, assumed_mask);

        let mask = ipv6::ipv6_wildcard_mask(80);
        let assumed_mask = "::ffff:ffff:ffff".parse::<Ipv6Addr>().unwrap().octets();
        assert_eq!(mask, assumed_mask);

        let network =
            ipv6::ipv6_network_address("2001:db8::1".parse::<Ipv6Addr>().unwrap().octets(), 64);
        let assumed_network = "2001:db8::".parse::<Ipv6Addr>().unwrap().octets();
        assert_eq!(network, assumed_network);

        let broadcast =
            ipv6::ipv6_broadcast_address("2001:db8::1".parse::<Ipv6Addr>().unwrap().octets(), 64);
        let assumed_broadcast = "2001:db8::ffff:ffff:ffff:ffff"
            .parse::<Ipv6Addr>()
            .unwrap()
            .octets();
        assert_eq!(broadcast, assumed_broadcast);
    }
}
