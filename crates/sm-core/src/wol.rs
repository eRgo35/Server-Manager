#[derive(Debug, thiserror::Error)]
pub enum WolError {
    #[error("invalid MAC address: {0}")]
    BadMac(String),
}

pub const WOL_PORTS: [u16; 2] = [9, 7];

pub fn parse_mac(s: &str) -> Result<[u8; 6], WolError> {
    let parts: Vec<&str> = s.split([':', '-']).collect();
    if parts.len() != 6 {
        return Err(WolError::BadMac(s.into()));
    }
    let mut out = [0u8; 6];
    for (i, p) in parts.iter().enumerate() {
        out[i] = u8::from_str_radix(p, 16).map_err(|_| WolError::BadMac(s.into()))?;
    }
    Ok(out)
}

pub fn magic_packet(mac: [u8; 6]) -> [u8; 102] {
    let mut p = [0u8; 102];
    for b in p.iter_mut().take(6) {
        *b = 0xFF;
    }
    for chunk in 0..16 {
        p[6 + chunk * 6..6 + chunk * 6 + 6].copy_from_slice(&mac);
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn packet_layout() {
        let mac = parse_mac("01:02:03:04:05:06").unwrap();
        let p = magic_packet(mac);
        assert_eq!(&p[0..6], &[0xFF; 6]);
        assert_eq!(&p[6..12], &mac);
        assert_eq!(&p[96..102], &mac);
        assert_eq!(p.len(), 102);
    }
    #[test]
    fn rejects_bad_mac() {
        assert!(parse_mac("zz:zz").is_err());
    }
    #[test]
    fn accepts_dash_form() {
        assert!(parse_mac("AA-BB-CC-DD-EE-FF").is_ok());
    }
}
