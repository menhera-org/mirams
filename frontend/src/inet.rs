
use serde::{Deserialize, Serialize};

use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

pub type RawAsn = u32;
pub type RawIpv4Addr = [u8; 4];
pub type RawIpv6Addr = [u8; 16];

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

pub fn ipv6_subnet_mask(prefix_len: u8) -> RawIpv6Addr {
    let mut mask = 0xffffffff_ffffffff_ffffffff_ffffffffu128;
    mask = mask.checked_shr(prefix_len as u32).unwrap_or(0);
    mask = !mask;
    let mask = mask.to_be_bytes();
    mask
}

pub fn ipv6_wildcard_mask(prefix_len: u8) -> RawIpv6Addr {
    let mut mask = 0xffffffff_ffffffff_ffffffff_ffffffffu128;
    mask = mask.checked_shr(prefix_len as u32).unwrap_or(0);
    let mask = mask.to_be_bytes();
    mask
}

/// First address in the network
pub fn ipv6_network_address(ip: RawIpv6Addr, prefix_len: u8) -> RawIpv6Addr {
    let mask = ipv6_subnet_mask(prefix_len);
    let mut addr = [0; 16];
    for i in 0..16 {
        addr[i] = ip[i] & mask[i];
    }
    addr
}

/// Last address in the network
pub fn ipv6_broadcast_address(ip: RawIpv6Addr, prefix_len: u8) -> RawIpv6Addr {
    let wildcard = ipv6_wildcard_mask(prefix_len);
    let mut addr = [0; 16];
    for i in 0..16 {
        addr[i] = ip[i] | wildcard[i];
    }
    addr
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Ipv4Prefix {
    prefix: RawIpv4Addr,
    prefix_len: i32,
}

impl Ipv4Prefix {
    pub fn new(prefix: &str, prefix_len: i32) -> Result<Self, String> {
        if prefix_len < 0 || prefix_len > 32 {
            return Err(format!("Invalid IPv4 prefix length: {}", prefix_len));
        }
        let ip = match prefix.trim().parse::<Ipv4Addr>() {
            Ok(ip) => ip,
            Err(e) => return Err(format!("Invalid IPv4 address: {}", e)),
        }.octets();
        let network = ipv4_network_address(ip, prefix_len as u8);
        if ip != network {
            return Err(format!("Invalid IPv4 prefix: {} is not a network address for prefix length {}", prefix, prefix_len));
        }
        Ok(Ipv4Prefix {
            prefix: ip,
            prefix_len,
        })
    }

    pub fn prefix(&self) -> String {
        Ipv4Addr::new(self.prefix[0], self.prefix[1], self.prefix[2], self.prefix[3]).to_string()
    }

    pub fn prefix_octets(&self) -> [u8; 4] {
        self.prefix
    }

    pub fn prefix_len(&self) -> i32 {
        self.prefix_len
    }

    pub fn contains(&self, ip: &Self) -> bool {
        if self.prefix_len > ip.prefix_len {
            return false;
        }
        let mask = ipv4_subnet_mask(self.prefix_len as u8);
        for i in 0..4 {
            if (self.prefix[i] & mask[i]) != (ip.prefix[i] & mask[i]) {
                return false;
            }
        }
        true
    }
}

impl Display for Ipv4Prefix {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.prefix(), self.prefix_len)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Ipv6Prefix {
    prefix: RawIpv6Addr,
    prefix_len: i32,
}

impl Ipv6Prefix {
    pub fn new(prefix: &str, prefix_len: i32) -> Result<Self, String> {
        if prefix_len < 0 || prefix_len > 128 {
            return Err(format!("Invalid IPv6 prefix length: {}", prefix_len));
        }
        let ip = match prefix.trim().parse::<Ipv6Addr>() {
            Ok(ip) => ip,
            Err(e) => return Err(format!("Invalid IPv6 address: {}", e)),
        }.octets();
        let network = ipv6_network_address(ip, prefix_len as u8);
        if ip != network {
            return Err(format!("Invalid IPv6 prefix: {} is not a network address for prefix length {}", prefix, prefix_len));
        }
        Ok(Ipv6Prefix {
            prefix: ip,
            prefix_len,
        })
    }

    pub fn prefix(&self) -> String {
        Ipv6Addr::from_bits(u128::from_be_bytes(self.prefix)).to_string()
    }

    pub fn prefix_octets(&self) -> [u8; 16] {
        self.prefix
    }

    pub fn prefix_len(&self) -> i32 {
        self.prefix_len
    }

    pub fn contains(&self, ip: &Self) -> bool {
        if self.prefix_len > ip.prefix_len {
            return false;
        }
        let mask = ipv6_subnet_mask(self.prefix_len as u8);
        for i in 0..16 {
            if (self.prefix[i] & mask[i]) != (ip.prefix[i] & mask[i]) {
                return false;
            }
        }
        true
    }
}

impl Display for Ipv6Prefix {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.prefix(), self.prefix_len)
    }
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

impl FromStr for ObjectVisibility {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Public" => Ok(ObjectVisibility::Public),
            "Private" => Ok(ObjectVisibility::Private),
            "public" => Ok(ObjectVisibility::Public),
            "private" => Ok(ObjectVisibility::Private),
            _ => Err(format!("Invalid visibility value: {}", s)),
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
