//! Lenient GFX CapabilitiesAdvertise parsing for clients with newer capability versions.

use ironrdp_pdu::rdp::vc::dvc::gfx::{CapabilitiesAdvertisePdu, CapabilitySet, ClientPdu};
use ironrdp_pdu::{decode, PduResult};
use tracing::{debug, warn};

const CAPS_ADVERTISE_PDU_TYPE: u16 = 0x0012;

/// Decode a client GFX PDU, with a fallback parser for CapabilitiesAdvertise.
pub(crate) fn decode_client_pdu(data: &[u8]) -> PduResult<ClientPdu> {
    match decode(data) {
        Ok(ClientPdu::CapabilitiesAdvertise(caps)) if caps.0.is_empty() => {
            if is_capabilities_advertise(data) {
                match parse_capabilities_advertise(data) {
                    Ok((sets, _)) if !sets.is_empty() => {
                        return Ok(ClientPdu::CapabilitiesAdvertise(CapabilitiesAdvertisePdu(sets)));
                    }
                    _ => {}
                }
            }
            Ok(ClientPdu::CapabilitiesAdvertise(caps))
        }
        Ok(pdu) => Ok(pdu),
        Err(strict_err) => {
            if !is_capabilities_advertise(data) {
                return Err(ironrdp_pdu::custom_err!("GfxDecode", strict_err));
            }
            match parse_capabilities_advertise(data) {
                Ok((sets, skipped)) => {
                    debug!(
                        sets = sets.len(),
                        skipped_unknown = skipped,
                        "GFX: parsed CapabilitiesAdvertise leniently"
                    );
                    Ok(ClientPdu::CapabilitiesAdvertise(CapabilitiesAdvertisePdu(sets)))
                }
                Err(e) => {
                    warn!(
                        error = %e,
                        "GFX: strict caps decode failed; lenient parse also failed"
                    );
                    Err(ironrdp_pdu::custom_err!("GfxDecode", strict_err))
                }
            }
        }
    }
}

pub(crate) fn is_capabilities_advertise(data: &[u8]) -> bool {
    data.len() >= 8 && u16::from_le_bytes([data[0], data[1]]) == CAPS_ADVERTISE_PDU_TYPE
}

/// Walk capability sets; skip unknown versions. Returns (sets, skipped_any_unknown).
pub(crate) fn parse_capabilities_advertise(data: &[u8]) -> Result<(Vec<CapabilitySet>, bool), String> {
    if !is_capabilities_advertise(data) {
        return Err("not a CapabilitiesAdvertise PDU".into());
    }

    let count = u32::from_le_bytes(data[4..8].try_into().map_err(|_| "short header")?) as usize;
    let mut offset = 8usize;
    let mut sets = Vec::with_capacity(count);
    let mut skipped = false;

    for i in 0..count {
        if offset + 8 > data.len() {
            warn!(i, count, offset, len = data.len(), "GFX caps: truncated capability set header");
            break;
        }
        let declared_len = u32::from_le_bytes(
            data[offset + 4..offset + 8]
                .try_into()
                .map_err(|_| "bad data_len")?,
        ) as usize;
        // [MS-RDPEGFX] length is the full capability set size (version + pad + length + data).
        // Some encoders use body-only length after the 8-byte header; accept both.
        let total = capability_set_total_size(declared_len, offset, data.len());
        if total.is_none() {
            warn!(
                i,
                declared_len,
                offset,
                len = data.len(),
                "GFX caps: truncated capability set body"
            );
            break;
        }
        let total = total.unwrap();
        let cap_bytes = &data[offset..offset + total];
        match decode(cap_bytes) {
            Ok(cap) => sets.push(cap),
            Err(e) => {
                skipped = true;
                let version = u16::from_le_bytes([cap_bytes[0], cap_bytes[1]]);
                debug!(version, error = %e, "GFX caps: skipped unknown capability set version");
            }
        }
        offset += total;
    }

    Ok((sets, skipped))
}

/// Resolve how many bytes belong to one capability set starting at `offset`.
fn capability_set_total_size(declared_len: usize, offset: usize, buf_len: usize) -> Option<usize> {
    if declared_len >= 8 && offset + declared_len <= buf_len {
        return Some(declared_len);
    }
    if declared_len > 0 && offset + 8 + declared_len <= buf_len {
        return Some(8 + declared_len);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use ironrdp_core::encode_vec;
    use ironrdp_pdu::rdp::vc::dvc::gfx::{CapabilitiesV107Flags, CapabilitySet};

    #[test]
    fn strict_roundtrip_v10_7() {
        let set = CapabilitySet::V10_7 {
            flags: CapabilitiesV107Flags::empty(),
        };
        let pdu = ClientPdu::CapabilitiesAdvertise(CapabilitiesAdvertisePdu(vec![set]));
        let bytes = encode_vec(&pdu).expect("encode");
        let decoded = decode_client_pdu(&bytes).expect("decode");
        assert!(matches!(decoded, ClientPdu::CapabilitiesAdvertise(_)));
    }

    #[test]
    fn lenient_skips_unknown_version_body_length() {
        let mut buf = vec![0x12, 0x00, 0x00, 0x00, 1, 0, 0, 0];
        buf.extend_from_slice(&0x9999u16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&4u32.to_le_bytes());
        buf.extend_from_slice(&[0u8, 0, 0, 0]);

        let (sets, skipped) = parse_capabilities_advertise(&buf).expect("parse");
        assert!(skipped);
        assert!(sets.is_empty());
    }

    #[test]
    fn lenient_skips_unknown_version_ms_total_length() {
        let mut buf = vec![0x12, 0x00, 0x00, 0x00, 1, 0, 0, 0];
        buf.extend_from_slice(&0x9999u16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        let total_set = 12u32; // 8-byte header + 4-byte body
        buf.extend_from_slice(&total_set.to_le_bytes());
        buf.extend_from_slice(&[0u8, 0, 0, 0]);

        let (sets, skipped) = parse_capabilities_advertise(&buf).expect("parse");
        assert!(skipped);
        assert!(sets.is_empty());
    }
}
