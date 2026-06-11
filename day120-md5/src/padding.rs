use bitvec::{self, order::Msb0, prelude::*, vec::BitVec};

#[derive(Debug)]
struct Message {
    bytes: BitVec<u8, Msb0>,
    orig_len: u64,
}

impl Message {
    fn add_padding_bits(&mut self) {
        self.bytes.push(true);
        while self.bytes.len() % 512 != 448 {
            self.bytes.push(false);
        }
    }
    fn append_message_len(&mut self) {
        assert!(self.bytes.len() % 512 == 448);

        let lower = self.orig_len as u32;
        let upper = (self.orig_len >> 32) as u32;

        self.bytes.extend_from_bitslice(lower.view_bits::<Msb0>());
        self.bytes.extend_from_bitslice(upper.view_bits::<Msb0>());
    }
}

#[test]
fn test_zero_bit_message() {
    let mut msg: Message = Message {
        bytes: BitVec::new(),
        orig_len: 0,
    };

    assert_eq!(0, msg.orig_len);
    assert_eq!(0, msg.bytes.len());

    msg.add_padding_bits();

    assert_eq!(0, msg.orig_len);
    assert_eq!(448, msg.bytes.len());

    msg.append_message_len();

    assert_eq!(0, msg.orig_len);
    assert_eq!(512, msg.bytes.len());

    let len = msg.bytes.len();
    let ms32 = msg.bytes[len - 32..].load_be::<u32>();
    let ls32 = msg.bytes[len - 64..len - 32].load_be::<u32>();

    assert_eq!(0, ms32);
    assert_eq!(0, ls32);
}

#[test]
fn test_one_bit_message() {
    let mut msg: Message = Message {
        bytes: bitvec![u8, Msb0; 1; 1],
        orig_len: 1,
    };

    assert_eq!(1, msg.orig_len);
    assert_eq!(1, msg.bytes.len());

    msg.add_padding_bits();

    assert_eq!(1, msg.orig_len);
    assert_eq!(448, msg.bytes.len());

    msg.append_message_len();

    assert_eq!(1, msg.orig_len);
    assert_eq!(512, msg.bytes.len());

    let len = msg.bytes.len();
    let ms32 = msg.bytes[len - 32..].load_be::<u32>();
    let ls32 = msg.bytes[len - 64..len - 32].load_be::<u32>();

    assert_eq!(0, ms32);
    assert_eq!(1, ls32);
}

#[test]
fn test_447_bit_message() {
    let mut msg: Message = Message {
        bytes: bitvec![u8, Msb0; 1; 447],
        orig_len: 447,
    };

    assert_eq!(447, msg.orig_len);
    assert_eq!(447, msg.bytes.len());

    msg.add_padding_bits();

    assert_eq!(447, msg.orig_len);
    assert_eq!(448, msg.bytes.len());

    msg.append_message_len();

    assert_eq!(447, msg.orig_len);
    assert_eq!(512, msg.bytes.len());

    let len = msg.bytes.len();
    let ms32 = msg.bytes[len - 32..].load_be::<u32>();
    let ls32 = msg.bytes[len - 64..len - 32].load_be::<u32>();

    assert_eq!(0, ms32);
    assert_eq!(447, ls32);
}

#[test]
fn test_448_bit_message() {
    let mut msg: Message = Message {
        bytes: bitvec![u8, Msb0; 1; 448],
        orig_len: 448,
    };

    assert_eq!(448, msg.orig_len);
    assert_eq!(448, msg.bytes.len());

    msg.add_padding_bits();

    assert_eq!(448, msg.orig_len);
    assert_eq!(448 + 512, msg.bytes.len());

    msg.append_message_len();

    assert_eq!(448, msg.orig_len);
    assert_eq!(1024, msg.bytes.len());

    let len = msg.bytes.len();
    let ms32 = msg.bytes[len - 32..].load_be::<u32>();
    let ls32 = msg.bytes[len - 64..len - 32].load_be::<u32>();

    assert_eq!(0, ms32);
    assert_eq!(448, ls32);
}

#[test]
fn test_large_message() {
    let mut msg: Message = Message {
        bytes: bitvec![u8, Msb0; 1; 5_000_000_000],
        orig_len: 5_000_000_000,
    };

    msg.add_padding_bits();
    msg.append_message_len();

    assert_eq!(5_000_000_000, msg.orig_len);
    assert!(msg.bytes.len().is_multiple_of(512));

    let len = msg.bytes.len();
    let ms32 = msg.bytes[len - 32..].load_be::<u32>();
    let ls32 = msg.bytes[len - 64..len - 32].load_be::<u32>();

    assert_eq!((msg.orig_len >> 32) as u32, ms32);
    assert_eq!(msg.orig_len as u32, ls32);
}
