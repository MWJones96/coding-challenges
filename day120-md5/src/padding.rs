use bitvec::{self, order::Msb0, prelude::*, vec::BitVec};

pub fn add_padding(vec: &mut BitVec<u8, Msb0>) {
    let orin_len: u64 = vec.len() as u64;

    vec.push(true);
    while vec.len() % 512 != 448 {
        vec.push(false);
    }

    vec.extend_from_bitslice(orin_len.to_le_bytes().view_bits::<Msb0>());
}

#[test]
fn test_zero_bit_message() {
    let mut input = BitVec::new();

    assert_eq!(0, input.len());

    add_padding(&mut input);

    assert_eq!(512, input.len());

    let len = input.len();
    let orig_len: u64 = input[len - 64..].load_le::<u64>();

    assert_eq!(0, orig_len);
}

#[test]
fn test_one_bit_message() {
    let mut input = bitvec![u8, Msb0; 1; 1];

    assert_eq!(1, input.len());

    add_padding(&mut input);

    assert_eq!(512, input.len());

    let len = input.len();
    let orig_len: u64 = input[len - 64..].load_le::<u64>();

    assert_eq!(1, orig_len);
}

#[test]
fn test_447_bit_message() {
    let mut input = bitvec![u8, Msb0; 1; 447];

    assert_eq!(447, input.len());

    add_padding(&mut input);

    assert_eq!(512, input.len());

    let len = input.len();
    let orig_len: u64 = input[len - 64..].load_le::<u64>();

    assert_eq!(447, orig_len);
}

#[test]
fn test_448_bit_message() {
    let mut input = bitvec![u8, Msb0; 1; 448];

    assert_eq!(448, input.len());

    add_padding(&mut input);

    assert_eq!(1024, input.len());

    let len = input.len();
    let orig_len: u64 = input[len - 64..].load_le::<u64>();

    assert_eq!(448, orig_len);
}

#[test]
fn test_large_message() {
    let mut input = bitvec![u8, Msb0; 1; 5_000_000_000];

    assert_eq!(5_000_000_000, input.len());

    add_padding(&mut input);

    assert!(input.len().is_multiple_of(512));

    let len = input.len();
    let orig_len: u64 = input[len - 64..].load_le::<u64>();

    assert_eq!(5_000_000_000, orig_len);
}
