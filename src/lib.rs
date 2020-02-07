#![no_std]

use bitbuf::{BitBuf, BitBufMut, CopyError};
use core::ops::Deref;

macro_rules! offset {
    (1) => {
        0
    };
    (2) => {
        2u16.pow(7)
    };
    (3) => {
        offset!(2) as u32 + 2u32.pow(14)
    };
    (4) => {
        offset!(3) as u32 + 2u32.pow(21)
    };
    (5) => {
        offset!(4) as u64 + 2u64.pow(28)
    };
    (6) => {
        offset!(5) + 2u64.pow(35)
    };
    (7) => {
        offset!(6) + 2u64.pow(42)
    };
    (8) => {
        offset!(7) + 2u64.pow(49)
    };
    (9) => {
        offset!(8) + 2u64.pow(56)
    };
}

fn encode_len(n: u64) -> u8 {
    match n {
        n if n < offset!(2) as u64 => 1,
        n if n < offset!(3) as u64 => 2,
        n if n < offset!(4) as u64 => 3,
        n if n < offset!(5) => 4,
        n if n < offset!(6) => 5,
        n if n < offset!(7) => 6,
        n if n < offset!(8) => 7,
        n if n < offset!(9) => 8,
        _ => 9,
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
        buf.push(true).unwrap();
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
        buf.put(&input.to_le_bytes(), len).unwrap();
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
            }
            len += 1;
            if len == 9 {
                break;
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
            _ => return Err(Error::TooLong),
        };
        let mut data = [0u8; 8];
        buf.copy_to_slice(&mut data, len)?;
        Ok(u64::from_le_bytes(data))
    }
}
