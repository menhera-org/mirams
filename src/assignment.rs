use crate::{types::ObjectVisibility, Error, ErrorKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    Asn,
    Ipv4,
    Ipv6,
}

impl ResourceType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Asn => "asn",
            Self::Ipv4 => "ipv4",
            Self::Ipv6 => "ipv6",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "resource_type", rename_all = "lowercase")]
pub enum AssignmentRange {
    Asn {
        asn_from: u32,
        asn_to: u32,
    },
    Ipv4 {
        ipv4_prefix: [u8; 4],
        ipv4_prefix_len: i32,
    },
    Ipv6 {
        ipv6_prefix: [u8; 16],
        ipv6_prefix_len: i32,
    },
}

impl AssignmentRange {
    pub fn resource_type(&self) -> ResourceType {
        match self {
            Self::Asn { .. } => ResourceType::Asn,
            Self::Ipv4 { .. } => ResourceType::Ipv4,
            Self::Ipv6 { .. } => ResourceType::Ipv6,
        }
    }

    /// Big-endian bounds allow the database to compare all ranges exactly.
    pub(crate) fn bounds(&self) -> Result<([u8; 16], [u8; 16]), Error> {
        match *self {
            Self::Asn { asn_from, asn_to } => {
                if asn_from > asn_to {
                    return Err(Error::new(ErrorKind::InvalidInput, "Invalid ASN range"));
                }
                Ok((
                    (asn_from as u128).to_be_bytes(),
                    (asn_to as u128).to_be_bytes(),
                ))
            }
            Self::Ipv4 {
                ipv4_prefix,
                ipv4_prefix_len,
            } => {
                crate::ipv4::validate_prefix(ipv4_prefix, ipv4_prefix_len)?;
                let last = crate::ipv4::ipv4_broadcast_address(ipv4_prefix, ipv4_prefix_len as u8);
                Ok((
                    (u32::from_be_bytes(ipv4_prefix) as u128).to_be_bytes(),
                    (u32::from_be_bytes(last) as u128).to_be_bytes(),
                ))
            }
            Self::Ipv6 {
                ipv6_prefix,
                ipv6_prefix_len,
            } => {
                crate::ipv6::validate_prefix(ipv6_prefix, ipv6_prefix_len)?;
                Ok((
                    ipv6_prefix,
                    crate::ipv6::ipv6_broadcast_address(ipv6_prefix, ipv6_prefix_len as u8),
                ))
            }
        }
    }

    pub(crate) fn contains(&self, child: &Self) -> Result<bool, Error> {
        let (start, end) = self.bounds()?;
        let (child_start, child_end) = child.bounds()?;
        Ok(self.resource_type() == child.resource_type()
            && start <= child_start
            && child_end <= end)
    }
}

/// A root or a child assignment. Any assignment can have further children.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assignment {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub parent_id: Option<i32>,
    pub name: String,
    pub description: String,
    pub visibility: ObjectVisibility,
    #[serde(flatten)]
    pub range: AssignmentRange,
}

pub trait AssignmentStore {
    fn get_assignment(&self, id: i32) -> Result<Assignment, Error>;
    /// List immediate children, or roots when parent_id is None.
    fn get_assignments(
        &self,
        parent_id: Option<i32>,
        resource_type: Option<ResourceType>,
    ) -> Result<Vec<Assignment>, Error>;
    /// Ancestors in order from the root to the immediate parent.
    fn get_ancestors(&self, id: i32) -> Result<Vec<Assignment>, Error>;
    /// An assignment is public only when it and every ancestor are public.
    fn is_public(&self, id: i32) -> Result<bool, Error>;
    fn create_assignment(&self, assignment: &Assignment) -> Result<i32, Error>;
    fn update_assignment(
        &self,
        id: i32,
        name: &str,
        description: &str,
        visibility: Option<ObjectVisibility>,
    ) -> Result<(), Error>;
    /// Move the entire subtree; None makes it a root assignment.
    fn move_assignment(&self, id: i32, parent_id: Option<i32>) -> Result<(), Error>;
    /// Destinations that satisfy containment, overlap, and cycle constraints.
    fn get_move_targets(&self, id: i32) -> Result<Vec<Assignment>, Error>;
    /// Delete an assignment and its complete subtree.
    fn delete_assignment(&self, id: i32) -> Result<(), Error>;
}
