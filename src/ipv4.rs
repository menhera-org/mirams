
use crate::types::Error;
use crate::types::ObjectVisibility;

use serde::{Serialize, Deserialize};

pub type RawIpv4Addr = [u8; 4];

pub fn ipv4_subnet_mask(prefix_len: u8) -> RawIpv4Addr {
    let mut mask = 0xffffffffu32;
    mask = mask.checked_shr(prefix_len as u32).unwrap_or(0);
    mask = !mask;
    let mask = mask.to_be_bytes();
    mask
}

pub fn ipv4_wildcard_mask(prefix_len: u8) -> RawIpv4Addr {
    let mut mask = 0xffffffffu32;
    mask = mask.checked_shr(prefix_len as u32).unwrap_or(0);
    let mask = mask.to_be_bytes();
    mask
}

/// First address in the network
pub fn ipv4_network_address(ip: RawIpv4Addr, prefix_len: u8) -> RawIpv4Addr {
    let mask = ipv4_subnet_mask(prefix_len);
    let mut addr = [0; 4];
    for i in 0..4 {
        addr[i] = ip[i] & mask[i];
    }
    addr
}

/// Last address in the network
pub fn ipv4_broadcast_address(ip: RawIpv4Addr, prefix_len: u8) -> RawIpv4Addr {
    let wildcard = ipv4_wildcard_mask(prefix_len);
    let mut addr = [0; 4];
    for i in 0..4 {
        addr[i] = ip[i] | wildcard[i];
    }
    addr
}

/// IPv4 assignment space. Can contain multiple pools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentSpaceIpv4 {
    #[serde(default)]
    pub id: i32,

    /// Assignment Space name
    pub name: String,

    /// Document actual usage ratio, purpose, etc.
    pub description: String,

    /// Visibility of the assignment space
    pub space_visibility: ObjectVisibility,

    /// IPv4 prefix of the assignment space, in big-endian bit and byte order
    pub ipv4_prefix: [u8; 4],

    /// Length of the IPv4 prefix
    pub ipv4_prefix_len: i32,
}

/// IPv4 assignment pool. Can contain multiple assignments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentPoolIpv4 {
    #[serde(default)]
    pub id: i32,

    /// Parent assignment space ID
    pub assignment_space_id: i32,

    /// Assignment pool name
    pub name: String,

    /// Document actual usage ratio, purpose, etc.
    pub description: String,

    /// Visibility of the assignment pool
    pub pool_visibility: ObjectVisibility,

    /// IPv4 prefix of the assignment pool, in big-endian bit and byte order
    pub ipv4_prefix: [u8; 4],

    /// Length of the IPv4 prefix
    pub ipv4_prefix_len: i32,
}

/// IPv4 assignment to a specific entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentIpv4 {
    #[serde(default)]
    pub id: i32,

    /// Parent assignment pool ID
    pub assignment_pool_id: i32,

    /// Assignment name
    pub name: String,

    /// Document actual usage ratio, purpose, etc.
    pub description: String,

    /// Assignment visibility
    pub assignment_visibility: ObjectVisibility,

    /// IPv4 prefix of the assignment, in big-endian bit and byte order
    pub ipv4_prefix: [u8; 4],

    /// Length of the IPv4 prefix
    pub ipv4_prefix_len: i32,
}


pub trait Ipv4AssignmentStore {
    /// Get an assignment space by ID
    fn get_space(&self, space_id: i32) -> Result<AssignmentSpaceIpv4, Error>;

    /// Get all assignment spaces
    fn get_spaces(&self) -> Result<Vec<AssignmentSpaceIpv4>, Error>;

    /// Create a new assignment space
    /// Returns the ID of the new assignment space
    /// ID in input is ignored
    fn create_space(&self, space: &AssignmentSpaceIpv4) -> Result<i32, Error>;

    /// Update metadata for an assignment space
    fn update_space(&self, id: i32, name: &str, description: &str) -> Result<(), Error>;

    /// Delete an assignment space
    /// Also deletes all pools and assignments in the space
    fn delete_space(&self, space_id: i32) -> Result<(), Error>;

    /// Get an assignment pool by ID
    fn get_pool(&self, pool_id: i32) -> Result<AssignmentPoolIpv4, Error>;

    /// Get all assignment pools in a space
    fn get_pools(&self, space_id: i32) -> Result<Vec<AssignmentPoolIpv4>, Error>;

    /// Create a new assignment pool
    /// Returns the ID of the new assignment pool
    /// ID in input is ignored
    fn create_pool(&self, pool: &AssignmentPoolIpv4) -> Result<i32, Error>;

    /// Update metadata for an assignment pool
    fn update_pool(&self, id: i32, name: &str, description: &str) -> Result<(), Error>;

    /// Delete an assignment pool
    /// Also deletes all assignments in the pool
    fn delete_pool(&self, pool_id: i32) -> Result<(), Error>;

    /// Get an assignment by ID
    fn get_assignment(&self, assignment_id: i32) -> Result<AssignmentIpv4, Error>;

    /// Get all assignments in a pool
    fn get_assignments(&self, pool_id: i32) -> Result<Vec<AssignmentIpv4>, Error>;

    /// Create a new assignment
    /// Returns the ID of the new assignment
    /// ID in input is ignored
    fn create_assignment(&self, assignment: &AssignmentIpv4) -> Result<i32, Error>;

    /// Update metadata for an assignment
    fn update_assignment(&self, id: i32, name: &str, description: &str) -> Result<(), Error>;

    /// Delete an assignment
    fn delete_assignment(&self, assignment_id: i32) -> Result<(), Error>;
}
