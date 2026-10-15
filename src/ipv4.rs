use crate::types::Error;

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

/// Reject invalid lengths and addresses with host bits before doing database work.
pub(crate) fn validate_prefix(prefix: RawIpv4Addr, prefix_len: i32) -> Result<(), Error> {
    if !(0..=32).contains(&prefix_len) {
        return Err(Error::new(
            crate::ErrorKind::InvalidInput,
            "Invalid IPV4 prefix length",
        ));
    }
    if prefix != ipv4_network_address(prefix, prefix_len as u8) {
        return Err(Error::new(
            crate::ErrorKind::InvalidInput,
            "IPV4 prefix must be a network address",
        ));
    }
    Ok(())
}
