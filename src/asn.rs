
use crate::types::Error;
use crate::types::ObjectVisibility;

use serde::{Serialize, Deserialize};


/// ASN assignment space. Can contain multiple pools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentSpaceAsn {
    #[serde(default)]
    pub id: i32,

    /// Assignment Space name
    pub name: String,

    /// Document actual usage ratio, purpose, etc.
    pub description: String,

    /// Visibility of the assignment space
    pub space_visibility: ObjectVisibility,

    /// Start ASN of the assignment space
    pub asn_from: u32,

    /// End ASN of the assignment space
    pub asn_to: u32,
}

/// ASN assignment pool. Can contain multiple assignments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentPoolAsn {
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

    /// Start ASN of the assignment pool
    pub asn_from: u32,

    /// End ASN of the assignment pool
    pub asn_to: u32,
}

/// ASN assignment to a specific entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentAsn {
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

    /// Assigned ASN
    pub asn: u32,
}


pub trait AsnAssignmentStore {
    /// Get an assignment space by ID
    fn get_space(&self, space_id: i32) -> Result<AssignmentSpaceAsn, Error>;

    /// Get all assignment spaces
    fn get_spaces(&self) -> Result<Vec<AssignmentSpaceAsn>, Error>;

    /// Create a new assignment space
    /// Returns the ID of the new assignment space
    /// ID in input is ignored
    fn create_space(&self, space: &AssignmentSpaceAsn) -> Result<i32, Error>;

    /// Update metadata for an assignment space
    fn update_space(&self, id: i32, name: &str, description: &str) -> Result<(), Error>;

    /// Delete an assignment space
    /// Also deletes all pools and assignments in the space
    fn delete_space(&self, space_id: i32) -> Result<(), Error>;

    /// Get an assignment pool by ID
    fn get_pool(&self, pool_id: i32) -> Result<AssignmentPoolAsn, Error>;

    /// Get all assignment pools in a space
    fn get_pools(&self, space_id: i32) -> Result<Vec<AssignmentPoolAsn>, Error>;

    /// Create a new assignment pool
    /// Returns the ID of the new assignment pool
    /// ID in input is ignored
    fn create_pool(&self, pool: &AssignmentPoolAsn) -> Result<i32, Error>;

    /// Update metadata for an assignment pool
    fn update_pool(&self, id: i32, name: &str, description: &str) -> Result<(), Error>;

    /// Delete an assignment pool
    /// Also deletes all assignments in the pool
    fn delete_pool(&self, pool_id: i32) -> Result<(), Error>;

    /// Get an assignment by ID
    fn get_assignment(&self, assignment_id: i32) -> Result<AssignmentAsn, Error>;

    /// Get all assignments in a pool
    fn get_assignments(&self, pool_id: i32) -> Result<Vec<AssignmentAsn>, Error>;

    /// Create a new assignment
    /// Returns the ID of the new assignment
    /// ID in input is ignored
    fn create_assignment(&self, assignment: &AssignmentAsn) -> Result<i32, Error>;

    /// Update metadata for an assignment
    fn update_assignment(&self, id: i32, name: &str, description: &str) -> Result<(), Error>;

    /// Delete an assignment
    fn delete_assignment(&self, assignment_id: i32) -> Result<(), Error>;
}
