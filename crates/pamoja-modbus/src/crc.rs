//! CRC-16/MODBUS, the integrity check every Modbus RTU frame carries.

/// Computes the CRC-16/MODBUS of a byte slice.
///
/// This is the checksum a Modbus RTU frame ends with, and the reason a receiver can
/// trust a frame that arrived over a long, electrically noisy cable: the polynomial is
/// `0xA001` (the reflected form of `0x8005`), the initial value is `0xFFFF`, input and
/// output are reflected, and there is no final inversion. A frame appends the result
/// low byte first.
///
/// # Arguments
///
/// * `data` - the bytes to check: the unit address through the end of the PDU, that is,
///   the whole frame except the two CRC bytes themselves.
///
/// # Returns
///
/// The 16-bit CRC.
///
/// # Examples
///
/// ```
/// use pamoja_modbus::crc16;
///
/// // The standard CRC-16/MODBUS check value over the ASCII digits "123456789".
/// assert_eq!(crc16(b"123456789"), 0x4B37);
/// ```
pub fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= u16::from(byte);
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_standard_check_value() {
        assert_eq!(crc16(b"123456789"), 0x4B37);
    }

    #[test]
    fn matches_a_known_read_request_frame() {
        // The classic read-holding-registers example: the CRC of the frame body
        // 01 03 00 00 00 02 is 0x0BC4, which the wire carries as C4 0B.
        assert_eq!(crc16(&[0x01, 0x03, 0x00, 0x00, 0x00, 0x02]), 0x0BC4);
    }

    #[test]
    fn matches_the_spec_read_request_frame() {
        // The Modbus specification's read-holding-registers example, unit 0x11.
        assert_eq!(crc16(&[0x11, 0x03, 0x00, 0x6B, 0x00, 0x03]), 0x8776);
    }

    #[test]
    fn an_empty_slice_is_the_initial_value() {
        assert_eq!(crc16(&[]), 0xFFFF);
    }
}
