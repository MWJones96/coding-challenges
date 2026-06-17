use bitvec::field::BitField;

use crate::process::{
    ComputeHash, get_bits_from_bytes,
    padding::{Endian, add_padding},
};

pub struct SHA1;
impl ComputeHash for SHA1 {
    fn process(msg: Vec<u8>) -> String {
        let mut h0: u32 = 0x67452301;
        let mut h1: u32 = 0xEFCDAB89;
        let mut h2: u32 = 0x98BADCFE;
        let mut h3: u32 = 0x10325476;
        let mut h4: u32 = 0xC3D2E1F0;

        let mut msg = get_bits_from_bytes(msg);
        add_padding(&mut msg, Endian::Big);

        for chunk in msg.chunks(512) {
            let mut w: [u32; 80] = [0; 80];
            for (i, chunk_i) in chunk.chunks(32).enumerate() {
                w[i] = chunk_i.load_be::<u32>();
            }

            for i in 16..80 {
                w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
            }

            let mut a = h0;
            let mut b = h1;
            let mut c = h2;
            let mut d = h3;
            let mut e = h4;

            for i in 0..80 {
                let mut f: u32 = 0;
                let mut k: u32 = 0;
                if i < 20 {
                    f = (b & c) | (!b & d);
                    k = 0x5A827999;
                } else if i < 40 {
                    f = b ^ c ^ d;
                    k = 0x6ED9EBA1;
                } else if i < 60 {
                    f = (b & c) | (b & d) | (c & d);
                    k = 0x8F1BBCDC;
                } else if i < 80 {
                    f = b ^ c ^ d;
                    k = 0xCA62C1D6;
                }

                let temp = a.rotate_left(5) + f + e + k + w[i];
                e = d;
                d = c;
                c = b.rotate_left(30);
                b = a;
                a = temp;
            }

            h0 += a;
            h1 += b;
            h2 += c;
            h3 += d;
            h4 += e;
        }

        format!("{:08x}{:08x}{:08x}{:08x}{:08x}", h0, h1, h2, h3, h4)
    }
}

#[test]
fn test_sha1() {
    let expected = "da39a3ee5e6b4b0d3255bfef95601890afd80709";
    assert_eq!(expected, SHA1::process("".as_bytes().into()));

    let expected = "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12";
    assert_eq!(
        expected,
        SHA1::process(
            "The quick brown fox jumps over the lazy dog"
                .as_bytes()
                .into()
        )
    );

    let expected = "de9f2c7fd25e1b3afad3e85a0bd17d9b100db4b3";
    assert_eq!(
        expected,
        SHA1::process(
            "The quick brown fox jumps over the lazy cog"
                .as_bytes()
                .into()
        )
    );
}
