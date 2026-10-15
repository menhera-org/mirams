use crate::types::Error;

pub type RawIpv6Addr = [u8; 16];

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

/// Reject invalid lengths and addresses with host bits before doing database work.
pub(crate) fn validate_prefix(prefix: RawIpv6Addr, prefix_len: i32) -> Result<(), Error> {
    if !(0..=128).contains(&prefix_len) {
        return Err(Error::new(
            crate::ErrorKind::InvalidInput,
            "Invalid IPV6 prefix length",
        ));
    }
    if prefix != ipv6_network_address(prefix, prefix_len as u8) {
        return Err(Error::new(
            crate::ErrorKind::InvalidInput,
            "IPV6 prefix must be a network address",
        ));
    }
    Ok(())
}
