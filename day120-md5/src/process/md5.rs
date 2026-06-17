use bitvec::field::BitField;

use crate::process::{ComputeHash, padding::Endian};

#[rustfmt::skip]
const SINE_TABLE: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x2441453,  0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x4881d05,  0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];

#[rustfmt::skip]
const INDEX_TABLE: [usize; 64] = [
    0, 1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15,
    1, 6, 11,  0,  5, 10, 15,  4,  9, 14,  3,  8, 13,  2,  7, 12, 
    5, 8, 11, 14,  1,  4,  7, 10, 13,  0,  3,  6,  9, 12, 15,  2, 
    0, 7, 14,  5, 12,  3, 10,  1,  8, 15,  6, 13,  4, 11,  2,  9,
];

#[rustfmt::skip]
const SHIFT_TABLE: [u8; 64] = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
    5,  9, 14, 20, 5,  9, 14, 20, 5,  9, 14, 20, 5,  9, 14, 20, 
    4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 
    6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
];

#[inline]
fn f(x: u32, y: u32, z: u32) -> u32 {
    (x & y) | (!x & z)
}

#[inline]
fn g(x: u32, y: u32, z: u32) -> u32 {
    (x & z) | (y & !z)
}

#[inline]
fn h(x: u32, y: u32, z: u32) -> u32 {
    x ^ y ^ z
}

#[inline]
fn i(x: u32, y: u32, z: u32) -> u32 {
    y ^ (x | !z)
}

#[inline]
fn l_rotate(bits: u32, amt: u8) -> u32 {
    let bits64: u64 = (bits as u64) << amt;
    let upper = (bits64 >> 32) as u32;
    let lower = bits64 as u32;

    upper | lower
}

#[inline]
fn update<F: Fn(u32, u32, u32) -> u32>(registers: [u32; 4], x: u32, s: u8, t: u32, f: F) -> u32 {
    let [a, b, c, d] = registers;
    b + l_rotate(a + f(b, c, d) + x + t, s)
}

pub struct MD5;
impl ComputeHash for MD5 {
    #[rustfmt::skip]
    fn process(msg: Vec<u8>) -> String {
        let mut aa: u32 = 0x67452301;
        let mut bb: u32 = 0xefcdab89;
        let mut cc: u32 = 0x98badcfe;
        let mut dd: u32 = 0x10325476;

        let mut msg = super::get_bits_from_bytes(msg);

        super::padding::add_padding(&mut msg, Endian::Little);

        for chunk in msg.chunks(512) {
            let x: Vec<u32> = chunk
                .chunks(32)
                .map(|chunk| chunk.load_le::<u32>())
                .collect::<_>();
            let x: [u32; 16] = x.try_into().expect("X must contain exactly 16 elements");

            let mut a = aa;
            let mut b = bb;
            let mut c = cc;
            let mut d = dd;

            for (i, op) in [f, g, h, i].iter().enumerate() {
                for j in 0..4 {
                    let idx: usize = i * 16 + j * 4;

                    a = update([a, b, c, d], x[INDEX_TABLE[idx]], SHIFT_TABLE[idx], SINE_TABLE[idx], op);
                    d = update([d, a, b, c], x[INDEX_TABLE[idx + 1]], SHIFT_TABLE[idx + 1], SINE_TABLE[idx + 1], op);
                    c = update([c, d, a, b], x[INDEX_TABLE[idx + 2]], SHIFT_TABLE[idx + 2], SINE_TABLE[idx + 2], op);
                    b = update([b, c, d, a], x[INDEX_TABLE[idx + 3]], SHIFT_TABLE[idx + 3], SINE_TABLE[idx + 3], op);
                }
            }

            aa += a;
            bb += b;
            cc += c;
            dd += d;
        }

        let mut builder = String::new();
        for byte in aa.to_le_bytes() {
            builder.push_str(&format!("{:02x}", byte));
        }
        for byte in bb.to_le_bytes() {
            builder.push_str(&format!("{:02x}", byte));
        }
        for byte in cc.to_le_bytes() {
            builder.push_str(&format!("{:02x}", byte));
        }
        for byte in dd.to_le_bytes() {
            builder.push_str(&format!("{:02x}", byte));
        }

        builder
    }
}

#[test]
fn test_md5() {
    let expected = "d41d8cd98f00b204e9800998ecf8427e";
    let output = MD5::process("".into());
    assert_eq!(expected, output);

    let expected = "0cc175b9c0f1b6a831c399e269772661";
    let output = MD5::process("a".into());
    assert_eq!(expected, output);

    let expected = "900150983cd24fb0d6963f7d28e17f72";
    let output = MD5::process("abc".into());
    assert_eq!(expected, output);

    let expected = "f96b697d7cb7938d525a2f31aaf161d0";
    let output = MD5::process("message digest".into());
    assert_eq!(expected, output);

    let expected = "c3fcd3d76192e4007dfb496cca67e13b";
    let output = MD5::process("abcdefghijklmnopqrstuvwxyz".into());
    assert_eq!(expected, output);

    let expected = "d174ab98d277d9f5a5611c2c9f419d9f";
    let output =
        MD5::process("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789".into());
    assert_eq!(expected, output);

    let expected = "57edf4a22be3c955ac49da2e2107b67a";
    let output = MD5::process(
        "12345678901234567890123456789012345678901234567890123456789012345678901234567890".into(),
    );
    assert_eq!(expected, output);
}
