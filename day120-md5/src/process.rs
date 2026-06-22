use bitvec::{array::BitArray, order::Msb0, vec::BitVec};

pub mod padding;

pub mod md5;
pub mod sha1;
pub mod sha256;
pub mod sha384;
pub mod sha512;

pub trait ComputeHash {
    fn process(msg: Vec<u8>) -> String;
}

pub trait Hmac {
    fn process_hmac(msg: Vec<u8>, key: Vec<u8>) -> String;
}

fn get_bits_from_bytes(bytes: &[u8]) -> BitVec<u8, Msb0> {
    bytes
        .iter()
        .flat_map(|&byte| {
            let bits = BitArray::<[u8; 1], Msb0>::new([byte]);
            bits.into_iter().collect::<Vec<bool>>()
        })
        .collect()
}

fn pad_key<CH: ComputeHash>(key: &mut Vec<u8>) {
    const BLOCK_LEN: usize = 64;

    if key.len() > BLOCK_LEN {
        *key = CH::process(key.clone()).into_bytes();
    } else if key.len() < BLOCK_LEN {
        while key.len() < BLOCK_LEN {
            key.push(0);
        }
    }
}
