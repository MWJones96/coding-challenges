use bitvec::field::BitField;

use crate::process::{
    ComputeHash, get_bits_from_bytes,
    padding::{Endian, add_padding},
};

#[rustfmt::skip]
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

pub struct SHA256;
impl ComputeHash for SHA256 {
    fn process(msg: Vec<u8>) -> String {
        let mut h0: u32 = 0x6a09e667;
        let mut h1: u32 = 0xbb67ae85;
        let mut h2: u32 = 0x3c6ef372;
        let mut h3: u32 = 0xa54ff53a;
        let mut h4: u32 = 0x510e527f;
        let mut h5: u32 = 0x9b05688c;
        let mut h6: u32 = 0x1f83d9ab;
        let mut h7: u32 = 0x5be0cd19;

        let mut msg = get_bits_from_bytes(msg);
        add_padding(&mut msg, Endian::Big);
        for chunk in msg.chunks(512) {
            let mut w: [u32; 64] = [0; 64];
            for (i, w_chunk) in chunk.chunks(32).enumerate() {
                w[i] = w_chunk.load_be::<u32>();
            }

            for i in 16..64 {
                let s0 =
                    (w[i - 15].rotate_right(7)) ^ (w[i - 15].rotate_right(18)) ^ (w[i - 15] >> 3);
                let s1 =
                    (w[i - 2].rotate_right(17)) ^ (w[i - 2].rotate_right(19)) ^ (w[i - 2] >> 10);
                w[i] = w[i - 16] + s0 + w[i - 7] + s1;
            }

            let mut a = h0;
            let mut b = h1;
            let mut c = h2;
            let mut d = h3;
            let mut e = h4;
            let mut f = h5;
            let mut g = h6;
            let mut h = h7;

            for i in 0..64 {
                let s1 = (e.rotate_right(6)) ^ (e.rotate_right(11)) ^ (e.rotate_right(25));
                let ch = (e & f) ^ (!e & g);
                let temp1 = h + s1 + ch + K[i] + w[i];
                let s0 = (a.rotate_right(2)) ^ (a.rotate_right(13)) ^ (a.rotate_right(22));
                let maj = (a & b) ^ (a & c) ^ (b & c);
                let temp2 = s0 + maj;

                h = g;
                g = f;
                f = e;
                e = d + temp1;
                d = c;
                c = b;
                b = a;
                a = temp1 + temp2;
            }

            h0 += a;
            h1 += b;
            h2 += c;
            h3 += d;
            h4 += e;
            h5 += f;
            h6 += g;
            h7 += h;
        }

        format!(
            "{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}",
            h0, h1, h2, h3, h4, h5, h6, h7
        )
    }
}

#[test]
fn test_sha256() {
    let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    assert_eq!(expected, SHA256::process("".as_bytes().into()));

    let expected = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    assert_eq!(expected, SHA256::process("abc".as_bytes().into()));

    let expected = "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1";
    assert_eq!(
        expected,
        SHA256::process(
            "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
                .as_bytes()
                .into()
        )
    );
}
