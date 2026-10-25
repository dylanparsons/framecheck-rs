use std::process;

const SYNC: [u8; 2] = [0xAA, 0x55];
const HEADER_LEN: usize = 5;
const CRC_LEN: usize = 2;

#[derive(Debug, PartialEq, Eq)]
struct Frame {
    message_type: u8,
    payload: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
enum FrameError {
    TooShort,
    InvalidSync,
    LengthMismatch,
    InvalidCrc,
    PayloadTooLarge,
}

fn crc16_ccitt_false(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;

    for &byte in data {
        crc ^= (byte as u16) << 8;

        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ 0x1021
            } else {
                crc << 1
            };
        }
    }

    crc
}

fn encode_frame(message_type: u8, payload: &[u8]) -> Result<Vec<u8>, FrameError> {
    let payload_len = u16::try_from(payload.len()).map_err(|_| FrameError::PayloadTooLarge)?;

    let mut frame = vec![SYNC[0], SYNC[1], message_type];
    frame.extend_from_slice(&payload_len.to_le_bytes());
    frame.extend_from_slice(payload);

    let crc = crc16_ccitt_false(&frame[2..]);
    frame.extend_from_slice(&crc.to_le_bytes());

    Ok(frame)
}

fn parse_frame(data: &[u8]) -> Result<Frame, FrameError> {
    if data.len() < HEADER_LEN + CRC_LEN {
        return Err(FrameError::TooShort);
    }

    if data[0..2] != SYNC {
        return Err(FrameError::InvalidSync);
    }

    let message_type = data[2];
    let payload_len = u16::from_le_bytes([data[3], data[4]]) as usize;
    let expected_len = HEADER_LEN + payload_len + CRC_LEN;

    if data.len() != expected_len {
        return Err(FrameError::LengthMismatch);
    }

    let crc_offset = data.len() - CRC_LEN;
    let received_crc = u16::from_le_bytes([data[crc_offset], data[crc_offset + 1]]);
    let calculated_crc = crc16_ccitt_false(&data[2..crc_offset]);

    if received_crc != calculated_crc {
        return Err(FrameError::InvalidCrc);
    }

    Ok(Frame {
        message_type,
        payload: data[HEADER_LEN..crc_offset].to_vec(),
    })
}

fn print_hex(bytes: &[u8]) {
    let hex = bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ");

    println!("{hex}");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.len() == 1 && args[0] == "--example" {
        let bytes = encode_frame(0x10, &[0xDE, 0xAD]).expect("example payload length fits in u16");

        println!("Generated frame:");
        print_hex(&bytes);
        return;
    }

    if args.is_empty() {
        eprintln!("Usage:");
        eprintln!("  framecheck-rs --example");
        eprintln!("  framecheck-rs \"AA 55 10 02 00 DE AD CRC1 CRC2\"");
        process::exit(2);
    }

    let input = args.join(" ");
    let bytes: Result<Vec<u8>, String> = input
        .split_whitespace()
        .map(|part| {
            let part = part.strip_prefix("0x").unwrap_or(part);
            u8::from_str_radix(part, 16).map_err(|_| format!("Invalid hex byte: {part}"))
        })
        .collect();

    let bytes = match bytes {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("Input error: {error}");
            process::exit(2);
        }
    };

    match parse_frame(&bytes) {
        Ok(frame) => {
            println!("Frame valid");
            println!("Message type: 0x{:02X}", frame.message_type);
            println!("Payload length: {}", frame.payload.len());
            print!("Payload: ");
            print_hex(&frame.payload);
        }
        Err(error) => {
            eprintln!("Invalid frame: {error:?}");
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_frame() {
        let bytes = encode_frame(0x10, &[0xDE, 0xAD]).expect("test payload length fits in u16");

        assert_eq!(
            parse_frame(&bytes),
            Ok(Frame {
                message_type: 0x10,
                payload: vec![0xDE, 0xAD],
            })
        );
    }

    #[test]
    fn rejects_invalid_sync() {
        let mut bytes = encode_frame(0x10, &[0x01]).expect("test payload length fits in u16");
        bytes[0] = 0x00;

        assert_eq!(parse_frame(&bytes), Err(FrameError::InvalidSync));
    }

    #[test]
    fn rejects_corrupted_crc() {
        let mut bytes = encode_frame(0x10, &[0x01, 0x02]).expect("test payload length fits in u16");
        let last = bytes.len() - 1;
        bytes[last] ^= 0xFF;

        assert_eq!(parse_frame(&bytes), Err(FrameError::InvalidCrc));
    }

    #[test]
    fn rejects_truncated_frame() {
        let mut bytes = encode_frame(0x10, &[0x01, 0x02]).expect("test payload length fits in u16");
        bytes.pop();

        assert_eq!(parse_frame(&bytes), Err(FrameError::LengthMismatch));
    }

    #[test]
    fn rejects_incomplete_header() {
        assert_eq!(parse_frame(&[0xAA, 0x55]), Err(FrameError::TooShort));
    }

    #[test]
    fn rejects_extra_bytes() {
        let mut bytes = encode_frame(0x10, &[0x01]).expect("test payload length fits in u16");
        bytes.push(0x00);

        assert_eq!(parse_frame(&bytes), Err(FrameError::LengthMismatch));
    }

    #[test]
    fn encoded_frame_has_valid_crc() {
        let bytes =
            encode_frame(0x42, &[0x01, 0x02, 0x03]).expect("test payload length fits in u16");

        assert!(parse_frame(&bytes).is_ok());
    }

    #[test]
    fn rejects_payload_larger_than_u16_length() {
        let payload = vec![0; usize::from(u16::MAX) + 1];

        assert_eq!(
            encode_frame(0x10, &payload),
            Err(FrameError::PayloadTooLarge)
        );
    }

    #[test]
    fn crc16_ccitt_false_matches_reference_vector() {
        assert_eq!(crc16_ccitt_false(b"123456789"), 0x29B1);
    }
}
