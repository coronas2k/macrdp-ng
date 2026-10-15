//! Lenient GFX CapabilitiesAdvertise parsing for clients with newer capability versions.

use ironrdp_pdu::rdp::vc::dvc::gfx::{CapabilitiesAdvertisePdu, CapabilitySet, ClientPdu};
use ironrdp_pdu::{decode, PduResult};
use tracing::{debug, warn};

const CAPS_ADVERTISE_PDU_TYPE: u16 = 0x0012;

/// Decode a client GFX PDU, with a fallback parser for CapabilitiesAdvertise.
pub fn decode_client_pdu(data: &[u8]) -> PduResult<ClientPdu> {
    match decode(data) {
        Ok(pdu) => Ok(pdu),
        Err(strict_err) => {
            if !is_capabilities_advertise(data) {
                return Err(strict_err);
            }
            let (sets, skipped) = parse_capabilities_advertise(data).map_err(|e| {
                warn!(error = %e, "GFX: strict caps decode failed; lenient parse also failed");
                strict_err
            })?;
            debug!(
                sets = sets.len(),
                skipped_unknown = skipped,
                "GFX: parsed CapabilitiesAdvertise leniently"
            );
            Ok(ClientPdu::CapabilitiesAdvertise(CapabilitiesAdvertisePdu(sets)))
        }
    }
}

pub fn is_capabilities_advertise(data: &[u8]) -> bool {
    data.len() >= 8 && u16::from_le_bytes([data[0], data[1]]) == CAPS_ADVERTISE_PDU_TYPE
}

/// Walk capability sets; skip unknown versions. Returns (sets, skipped_any_unknown).
pub fn parse_capabilities_advertise(data: &[u8]) -> Result<(Vec<CapabilitySet>, bool), String> {
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
        let data_len = u32::from_le_bytes(
            data[offset + 4..offset + 8]
                .try_into()
                .map_err(|_| "bad data_len")?,
        ) as usize;
        let total = 8 + data_len;
        if offset + total > data.len() {
            warn!(i, total, offset, len = data.len(), "GFX caps: truncated capability set body");
            break;
        }
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
    fn lenient_skips_unknown_version() {
        let mut buf = vec![0x12, 0x00, 0x00, 0x00, 1, 0, 0, 0];
        buf.extend_from_slice(&0x9999u16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&4u32.to_le_bytes());
        buf.extend_from_slice(&[0u8, 0, 0, 0]);

        let (sets, skipped) = parse_capabilities_advertise(&buf).expect("parse");
        assert!(skipped);
        assert!(sets.is_empty());
    }
}
