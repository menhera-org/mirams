
use serde::{Deserialize, Serialize};

use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};

pub fn format_asn_range(start: u32, end: u32) -> String {
    if start == end {
        format!("AS{}", start)
    } else {
        format!("AS{} - AS{}", start, end)
    }
}

pub fn format_ipv4_prefix(prefix: [u8; 4], prefix_len: i32) -> String {
    let ip = Ipv4Addr::new(prefix[0], prefix[1], prefix[2], prefix[3]);
    format!("{}/{}", ip, prefix_len)
}

pub fn format_ipv6_prefix(prefix: [u8; 16], prefix_len: i32) -> String {
    let ip = Ipv6Addr::from_bits(u128::from_be_bytes(prefix));
    format!("{}/{}", ip, prefix_len)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiResponse {
    pub error: Option<String>,

    #[serde(flatten)] 
    pub result: Option<ApiResponseVariant>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")] 
pub enum ApiResponseVariant {
    AsnAssignmentSpace(AssignmentSpaceAsn),
    AsnAssignmentPool(AssignmentPoolAsn),
    AsnAssignment(AssignmentAsn),
    Ipv4AssignmentSpace(AssignmentSpaceIpv4),
    Ipv4AssignmentPool(AssignmentPoolIpv4),
    Ipv4Assignment(AssignmentIpv4),
    Ipv6AssignmentSpace(AssignmentSpaceIpv6),
    Ipv6AssignmentPool(AssignmentPoolIpv6),
    Ipv6Assignment(AssignmentIpv6),

    AsnAssignmentSpaces(Vec<AssignmentSpaceAsn>),
    AsnAssignmentPools(Vec<AssignmentPoolAsn>),
    AsnAssignments(Vec<AssignmentAsn>),
    Ipv4AssignmentSpaces(Vec<AssignmentSpaceIpv4>),
    Ipv4AssignmentPools(Vec<AssignmentPoolIpv4>),
    Ipv4Assignments(Vec<AssignmentIpv4>),
    Ipv6AssignmentSpaces(Vec<AssignmentSpaceIpv6>),
    Ipv6AssignmentPools(Vec<AssignmentPoolIpv6>),
    Ipv6Assignments(Vec<AssignmentIpv6>),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[repr(i32)]
pub enum ObjectVisibility {
    /// Assignment visible to everyone
    Public = 0,

    /// Assignment only visible to logged-in users
    Private = 1,
}

impl TryFrom<i32> for ObjectVisibility {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ObjectVisibility::Public),
            1 => Ok(ObjectVisibility::Private),
            _ => Err(format!("Invalid visibility value: {}", value)),
        }
    }
}

impl Display for ObjectVisibility {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ObjectVisibility::Public => write!(f, "Public"),
            ObjectVisibility::Private => write!(f, "Private"),
        }
    }
}

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

/// IPv6 assignment space. Can contain multiple pools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentSpaceIpv6 {
    #[serde(default)]
    pub id: i32,

    /// Assignment Space name
    pub name: String,

    /// Document actual usage ratio, purpose, etc.
    pub description: String,

    /// Visibility of the assignment space
    pub space_visibility: ObjectVisibility,

    /// IPv6 prefix of the assignment space, in big-endian bit and byte order
    pub ipv6_prefix: [u8; 16],

    /// Length of the IPv6 prefix
    pub ipv6_prefix_len: i32,
}

/// IPv6 assignment pool. Can contain multiple assignments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentPoolIpv6 {
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

    /// IPv6 prefix of the assignment pool, in big-endian bit and byte order
    pub ipv6_prefix: [u8; 16],

    /// Length of the IPv6 prefix
    pub ipv6_prefix_len: i32,
}

/// IPv6 assignment to a specific entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentIpv6 {
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

    /// IPv6 prefix of the assignment, in big-endian bit and byte order
    pub ipv6_prefix: [u8; 16],

    /// Length of the IPv6 prefix
    pub ipv6_prefix_len: i32,
}
