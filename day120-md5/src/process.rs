use bitvec::{array::BitArray, order::Msb0, vec::BitVec};

pub mod md5;
pub mod padding;
pub mod sha256;

pub trait ComputeHash {
    fn process(msg: Vec<u8>) -> String;
}

fn get_bits_from_bytes(bytes: Vec<u8>) -> BitVec<u8, Msb0> {
    bytes
        .iter()
        .flat_map(|&byte| {
            let bits = BitArray::<[u8; 1], Msb0>::new([byte]);
            bits.into_iter().collect::<Vec<bool>>()
        })
        .collect()
}
