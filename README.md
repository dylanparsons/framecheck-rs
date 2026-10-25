# framecheck-rs

A Rust-based binary frame parser and protocol validation tool featuring CRC-16/CCITT-FALSE error checking.

`framecheck-rs` is a small systems-programming project focused on binary protocol handling, data validation, and defensive error checking. It demonstrates Rust error handling, byte-level parsing, and unit testing.

## Features

- Parse and validate binary protocol frames from hexadecimal input.
- Encode frames with a synchronization header, message type, payload length, payload, and CRC.
- Calculate CRC-16/CCITT-FALSE checksums.
- Validate synchronization bytes and payload lengths.
- Reject truncated frames, extra bytes, corrupted checksums, and oversized payloads.
- Unit tests covering valid frames and common error conditions.

## Protocol Format

Each frame uses the following layout:

| Field | Size | Description |
|---|---:|---|
| Sync | 2 bytes | Fixed header: `AA 55` |
| Message type | 1 byte | Identifies the message |
| Payload length | 2 bytes | Unsigned 16-bit little-endian length |
| Payload | Variable | Message data, up to 65,535 bytes |
| CRC | 2 bytes | CRC-16/CCITT-FALSE, little-endian |

The CRC is calculated over the message type, payload length, and payload. The synchronization bytes are excluded.

## Requirements

- Rust toolchain and Cargo

## Build and Test

Clone the repository and enter the project directory:

```bash
git clone https://github.com/dylanparsons/framecheck-rs.git
cd framecheck-rs
```

Build the project:

```bash
cargo build
```

Run the unit tests:

```bash
cargo test
```

Run the example:

```bash
cargo run -- --example
```

Expected output:

```text
Generated frame:
AA 55 10 02 00 DE AD 51 AA
```

## Validate a Frame

Pass a frame as hexadecimal bytes:

```bash
cargo run -- "AA 55 10 02 00 DE AD 51 AA"
```

A valid frame reports its message type and payload.

The parser also detects invalid synchronization bytes, length mismatches, incomplete frames, and CRC errors.

## Error Handling

The implementation uses explicit error variants for:

- Frames that are too short
- Invalid synchronization bytes
- Payload length mismatches
- Invalid CRC values
- Payloads exceeding the 16-bit length limit

## Implementation Details

- **Language:** Rust
- **Checksum:** CRC-16/CCITT-FALSE
- **Initial CRC value:** `0xFFFF`
- **Polynomial:** `0x1021`
- **Length encoding:** 16-bit little-endian
- **Testing:** Rust unit tests

## License

See [LICENSE](LICENSE) for license information.
