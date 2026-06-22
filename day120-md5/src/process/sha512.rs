use bitvec::field::BitField;

use crate::{
    process::{
        ComputeHash, get_bits_from_bytes,
        padding::add_padding_1024,
    },
};

#[rustfmt::skip]
const K: [u64; 80] = [
    0x428a2f98d728ae22, 0x7137449123ef65cd, 0xb5c0fbcfec4d3b2f, 0xe9b5dba58189dbbc, 0x3956c25bf348b538, 
    0x59f111f1b605d019, 0x923f82a4af194f9b, 0xab1c5ed5da6d8118, 0xd807aa98a3030242, 0x12835b0145706fbe, 
    0x243185be4ee4b28c, 0x550c7dc3d5ffb4e2, 0x72be5d74f27b896f, 0x80deb1fe3b1696b1, 0x9bdc06a725c71235, 
    0xc19bf174cf692694, 0xe49b69c19ef14ad2, 0xefbe4786384f25e3, 0x0fc19dc68b8cd5b5, 0x240ca1cc77ac9c65, 
    0x2de92c6f592b0275, 0x4a7484aa6ea6e483, 0x5cb0a9dcbd41fbd4, 0x76f988da831153b5, 0x983e5152ee66dfab, 
    0xa831c66d2db43210, 0xb00327c898fb213f, 0xbf597fc7beef0ee4, 0xc6e00bf33da88fc2, 0xd5a79147930aa725, 
    0x06ca6351e003826f, 0x142929670a0e6e70, 0x27b70a8546d22ffc, 0x2e1b21385c26c926, 0x4d2c6dfc5ac42aed, 
    0x53380d139d95b3df, 0x650a73548baf63de, 0x766a0abb3c77b2a8, 0x81c2c92e47edaee6, 0x92722c851482353b, 
    0xa2bfe8a14cf10364, 0xa81a664bbc423001, 0xc24b8b70d0f89791, 0xc76c51a30654be30, 0xd192e819d6ef5218, 
    0xd69906245565a910, 0xf40e35855771202a, 0x106aa07032bbd1b8, 0x19a4c116b8d2d0c8, 0x1e376c085141ab53, 
    0x2748774cdf8eeb99, 0x34b0bcb5e19b48a8, 0x391c0cb3c5c95a63, 0x4ed8aa4ae3418acb, 0x5b9cca4f7763e373, 
    0x682e6ff3d6b2b8a3, 0x748f82ee5defb2fc, 0x78a5636f43172f60, 0x84c87814a1f0ab72, 0x8cc702081a6439ec, 
    0x90befffa23631e28, 0xa4506cebde82bde9, 0xbef9a3f7b2c67915, 0xc67178f2e372532b, 0xca273eceea26619c, 
    0xd186b8c721c0c207, 0xeada7dd6cde0eb1e, 0xf57d4f7fee6ed178, 0x06f067aa72176fba, 0x0a637dc5a2c898a6, 
    0x113f9804bef90dae, 0x1b710b35131c471b, 0x28db77f523047d84, 0x32caab7b40c72493, 0x3c9ebe0a15c9bebc, 
    0x431d67c49c100d4c, 0x4cc5d4becb3e42b6, 0x597f299cfc657e2a, 0x5fcb6fab3ad6faec, 0x6c44198c4a475817
];

pub struct SHA512;
impl ComputeHash for SHA512 {
    fn process(msg: Vec<u8>) -> String {
        #[rustfmt::skip]
        let mut h_: [u64; 8] = [
            0x6a09e667f3bcc908, 0xbb67ae8584caa73b, 0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1,
            0x510e527fade682d1, 0x9b05688c2b3e6c1f, 0x1f83d9abfb41bd6b, 0x5be0cd19137e2179
        ];

        let mut msg = get_bits_from_bytes(&msg);
        add_padding_1024(&mut msg);
        for chunk in msg.chunks(1024) {
            let mut w: [u64; 80] = [0; 80];
            for (i, chunk_i) in chunk.chunks(64).enumerate() {
                w[i] = chunk_i.load_be::<u64>();
            }

            for i in 16..80 {
                let s0: u64 =
                    (w[i - 15].rotate_right(1)) ^ (w[i - 15].rotate_right(8)) ^ (w[i - 15] >> 7);
                let s1: u64 =
                    (w[i - 2].rotate_right(19)) ^ (w[i - 2].rotate_right(61)) ^ (w[i - 2] >> 6);
                w[i] = w[i - 16] + s0 + w[i - 7] + s1;
            }

            let mut a: u64 = h_[0];
            let mut b: u64 = h_[1];
            let mut c: u64 = h_[2];
            let mut d: u64 = h_[3];
            let mut e: u64 = h_[4];
            let mut f: u64 = h_[5];
            let mut g: u64 = h_[6];
            let mut h: u64 = h_[7];

            for i in 0..80 {
                let S1: u64 = (e.rotate_right(14)) ^ (e.rotate_right(18)) ^ (e.rotate_right(41));
                let ch: u64 = (e & f) ^ (!e & g);
                let temp1: u64 = h + S1 + ch + K[i] + w[i];
                let S0: u64 = (a.rotate_right(28)) ^ (a.rotate_right(34)) ^ (a.rotate_right(39));
                let maj: u64 = (a & b) ^ (a & c) ^ (b & c);
                let temp2: u64 = S0 + maj;

                h = g;
                g = f;
                f = e;
                e = d + temp1;
                d = c;
                c = b;
                b = a;
                a = temp1 + temp2;
            }

            h_[0] += a;
            h_[1] += b;
            h_[2] += c;
            h_[3] += d;
            h_[4] += e;
            h_[5] += f;
            h_[6] += g;
            h_[7] += h;
        }

        format!(
            "{:016x}{:016x}{:016x}{:016x}{:016x}{:016x}{:016x}{:016x}",
            h_[0], h_[1], h_[2], h_[3], h_[4], h_[5], h_[6], h_[7]
        )
    }
}

#[test]
fn test_sha512() {
    let expected = "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e";
    assert_eq!(expected, SHA512::process("".as_bytes().into()));

    let expected = "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f";
    assert_eq!(expected, SHA512::process("abc".as_bytes().into()));

    let expected = "839e2406db5ab261c3a88937225f655eddcbe54c705e5809a8cdfa85339f3bec55c4d829634d5beaabd84eb89f594b9698758329412ddfbcda38898ef6869a68";
    assert_eq!(
        expected,
        SHA512::process("sha512 is pretty neat!".as_bytes().into())
    );

    let expected = "8e959b75dae313da8cf4f72814fc143f8f7779c6eb9f7fa17299aeadb6889018501d289e4900f7e4331b99dec4b5433ac7d329eeb6dd26545e96e55b874be909";
    assert_eq!(
        expected,
        SHA512::process("abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu".as_bytes().into())
    );
}
