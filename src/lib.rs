//#![no_std]
//! Provides a prefix-based variable length quantity format
//! that can contain any u64, occupying between 1 and 9 bytes
//! and wasting one bit per byte in serialized form.
//! ```
//! use bitbuf::{BitBuf, BitBufMut};
//! use bitbuf_vlq::Vlq;
//! let mut data = [0u8; 8];
//!
//! // Very large number (requires 48 bits)
//! let val: u64 = 1389766487781;
//!
//! // Create a buffer handle to write into the array
//! let mut buf = BitBufMut::new(&mut data);
//!
//! // Create a variable-length quantity (from any Into<u64>)
//! let vlq: Vlq = Vlq::from(val);
//!
//! // Write the vlq data to the buffer
//! buf.put_aligned(&*vlq).unwrap();
//!
//! // Note the length of the written data
//! assert_eq!(buf.len(), 48);
//!
//! // Create a buffer to read the data back out
//! let mut buf = BitBuf::new(&mut data);
//!
//! // Note the value is preserved
//! assert_eq!(Vlq::read(&mut buf).unwrap(), val);
//!
//! // Use a smaller value
//! let val: u64 = 78;
//!
//! // Create a new buffer handle to write into the array
//! let mut buf = BitBufMut::new(&mut data);
//!
//! //. Create a new variable-length quantity
//! let vlq: Vlq = Vlq::from(val);
//!
//! // Write the vlq data to the buffer
//! buf.put_aligned(&*vlq).unwrap();
//!
//! // Note the shorter length of the written data
//! assert_eq!(buf.len(), 8);
//!
//! // Create a buffer to read the data back out
//! let mut buf = BitBuf::new(&mut data);
//!
//! // Note the value is preserved
//! assert_eq!(Vlq::read(&mut buf).unwrap(), val);
//! ```

use bitbuf::{BitBuf, BitBufMut, CopyError};
use core::ops::Deref;

fn encode_len(n: u64) -> u8 {
    if n < 2u64.pow(7) {
        0
    } else if n < 2u64.pow(14) {
        1
    } else if n < 2u64.pow(20) {
        2
    } else if n < 2u64.pow(28) {
        3
    } else if n < 2u64.pow(35) {
        4
    } else if n < 2u64.pow(42) {
        5
    } else if n < 2u64.pow(49) {
        6
    } else if n < 2u64.pow(56) {
        7
    } else {
        8
    }
}

fn decode_len(n: u8) -> u8 {
    n.leading_zeros() as u8 + 1
}

#[derive(Debug, PartialEq, Eq)]
pub struct Vlq([u8; 9]);

impl Deref for Vlq {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        let data: &[u8] = &self.0;
        &data[..decode_len(self.0[0]) as usize]
    }
}

impl<T: Into<u64>> From<T> for Vlq {
    fn from(input: T) -> Self {
        let input = input.into();
        let mut encoded = [0u8; 9];
        let mut buf = BitBufMut::new(&mut encoded);
        let len = encode_len(input);
        for _ in 0..len {
            buf.push(false).unwrap();
        }
        if len != 8 {
            buf.push(true).unwrap();
        }
        let len = match len {
            0 => 7,
            1 => 14,
            2 => 20,
            3 => 28,
            4 => 35,
            5 => 42,
            6 => 49,
            7 => 56,
            8 => 64,
            _ => panic!("determined invalid length"),
        };
        let mut bytes = input.to_le_bytes();
        for byte in &mut bytes {
            *byte = byte.reverse_bits();
        }
        buf.put(&bytes, len).unwrap();
        Vlq(encoded)
    }
}

#[derive(Debug)]
pub enum Error {
    TooLong,
    Buf(CopyError),
}

impl From<CopyError> for Error {
    fn from(input: CopyError) -> Self {
        Error::Buf(input)
    }
}

impl Vlq {
    pub fn read<'a>(buf: &mut BitBuf<'a>) -> Result<u64, Error> {
        let mut len = 0usize;
        while let Some(item) = buf.pop() {
            if item {
                break;
            } else {
                len += 1;
                if len == 8 {
                    break;
                }
            }
        }
        len = match len {
            0 => 7,
            1 => 14,
            2 => 20,
            3 => 28,
            4 => 35,
            5 => 42,
            6 => 49,
            7 => 56,
            8 => 64,
            _ => panic!("invalid length in Vlq read"),
        };
        let mut data = [0u8; 8];
        buf.copy_to_slice(&mut data, len)?;
        for byte in &mut data {
            *byte = byte.reverse_bits();
        }
        Ok(u64::from_le_bytes(data))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn read_write(value: u64, bytes: usize) {
        // Create backing storage
        let mut data = vec![0u8; bytes];

        // Create a buffer handle for writing
        let mut buf = BitBufMut::new(&mut data);

        // Create a vlq
        let vlq = Vlq::from(value);

        // Write vlq to buffer
        buf.put_aligned(&*vlq)
            .expect("writing vlq to buffer failed");

        // Ensure the correct byte length was written
        assert_eq!(buf.len(), bytes * 8);

        // Read vlq to ensure value is preserved
        assert_eq!(
            Vlq::read(&mut BitBuf::new(&data)).expect("reading vlq failed"),
            value
        );
    }

    #[test]
    fn u7_lower_bound() {
        read_write(0, 1);
    }

    #[test]
    fn u7_upper_bound() {
        read_write(2u64.pow(7) - 1, 1);
    }

    #[test]
    fn u14_lower_bound() {
        read_write(2u64.pow(7), 2);
    }

    #[test]
    fn u14_upper_bound() {
        read_write(2u64.pow(14) - 1, 2);
    }

    #[test]
    fn u20_lower_bound() {
        read_write(2u64.pow(14), 3);
    }

    #[test]
    fn u20_upper_bound() {
        read_write(2u64.pow(20) - 1, 3);
    }

    #[test]
    fn u28_lower_bound() {
        read_write(2u64.pow(20), 4);
    }

    #[test]
    fn u28_upper_bound() {
        read_write(2u64.pow(28) - 1, 4);
    }

    #[test]
    fn u35_lower_bound() {
        read_write(2u64.pow(28), 5);
    }

    #[test]
    fn u35_upper_bound() {
        read_write(2u64.pow(35) - 1, 5);
    }

    #[test]
    fn u42_lower_bound() {
        read_write(2u64.pow(35), 6);
    }

    #[test]
    fn u42_upper_bound() {
        read_write(2u64.pow(42) - 1, 6);
    }

    #[test]
    fn u49_lower_bound() {
        read_write(2u64.pow(42), 7);
    }

    #[test]
    fn u49_upper_bound() {
        read_write(2u64.pow(49) - 1, 7);
    }

    #[test]
    fn u56_lower_bound() {
        read_write(2u64.pow(49), 8);
    }

    #[test]
    fn u56_upper_bound() {
        read_write(2u64.pow(56) - 1, 8);
    }

    #[test]
    fn u64_lower_bound() {
        read_write(2u64.pow(56), 9);
    }

    #[test]
    fn u64_upper_bound() {
        read_write(core::u64::MAX, 9);
    }
}
