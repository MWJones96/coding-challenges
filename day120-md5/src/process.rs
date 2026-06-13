use bitvec::{array::BitArray, field::BitField, order::Msb0, vec::BitVec};

mod padding;

fn f(x: u32, y: u32, z: u32) -> u32 {
    (x & y) | (!x & z)
}
fn g(x: u32, y: u32, z: u32) -> u32 {
    (x & z) | (y & !z)
}
fn h(x: u32, y: u32, z: u32) -> u32 {
    x ^ y ^ z
}
fn i(x: u32, y: u32, z: u32) -> u32 {
    y ^ (x | !z)
}

fn update<F: Fn(u32, u32, u32) -> u32>(
    a: &mut u32,
    b: u32,
    c: u32,
    d: u32,
    x: u32,
    s: u8,
    t: u32,
    f: F,
) {
    *a = b.wrapping_add(l_rotate(
        (*a).wrapping_add(f(b, c, d))
            .wrapping_add(x)
            .wrapping_add(t),
        s,
    ));
}

fn l_rotate(bits: u32, amt: u8) -> u32 {
    let bits64: u64 = (bits as u64) << amt;
    let upper = (bits64 >> 32) as u32;
    let lower = bits64 as u32;

    upper | lower
}

pub fn process(msg: String) -> String {
    let mut A: u32 = 0x67452301;
    let mut B: u32 = 0xefcdab89;
    let mut C: u32 = 0x98badcfe;
    let mut D: u32 = 0x10325476;

    let mut bits: BitVec<u8, Msb0> = msg
        .as_bytes()
        .iter()
        .flat_map(|&byte| {
            let bits = BitArray::<[u8; 1], Msb0>::new([byte]);
            bits.into_iter().collect::<Vec<bool>>()
        })
        .collect();

    padding::add_padding(&mut bits);

    for chunk in bits.chunks(512) {
        let x: Vec<u32> = chunk
            .chunks(32)
            .map(|byte| byte.load_le::<u32>())
            .collect::<_>();
        let x: [u32; 16] = x.try_into().expect("X must contain exactly 16 elements");

        let mut a = A;
        let mut b = B;
        let mut c = C;
        let mut d = D;

        update(&mut a, b, c, d, x[0], 7, 0xd76aa478, f);
        update(&mut d, a, b, c, x[1], 12, 0xe8c7b756, f);
        update(&mut c, d, a, b, x[2], 17, 0x242070db, f);
        update(&mut b, c, d, a, x[3], 22, 0xc1bdceee, f);
        update(&mut a, b, c, d, x[4], 7, 0xf57c0faf, f);
        update(&mut d, a, b, c, x[5], 12, 0x4787c62a, f);
        update(&mut c, d, a, b, x[6], 17, 0xa8304613, f);
        update(&mut b, c, d, a, x[7], 22, 0xfd469501, f);
        update(&mut a, b, c, d, x[8], 7, 0x698098d8, f);
        update(&mut d, a, b, c, x[9], 12, 0x8b44f7af, f);
        update(&mut c, d, a, b, x[10], 17, 0xffff5bb1, f);
        update(&mut b, c, d, a, x[11], 22, 0x895cd7be, f);
        update(&mut a, b, c, d, x[12], 7, 0x6b901122, f);
        update(&mut d, a, b, c, x[13], 12, 0xfd987193, f);
        update(&mut c, d, a, b, x[14], 17, 0xa679438e, f);
        update(&mut b, c, d, a, x[15], 22, 0x49b40821, f);

        update(&mut a, b, c, d, x[1], 5, 0xf61e2562, g);
        update(&mut d, a, b, c, x[6], 9, 0xc040b340, g);
        update(&mut c, d, a, b, x[11], 14, 0x265e5a51, g);
        update(&mut b, c, d, a, x[0], 20, 0xe9b6c7aa, g);
        update(&mut a, b, c, d, x[5], 5, 0xd62f105d, g);
        update(&mut d, a, b, c, x[10], 9, 0x2441453, g);
        update(&mut c, d, a, b, x[15], 14, 0xd8a1e681, g);
        update(&mut b, c, d, a, x[4], 20, 0xe7d3fbc8, g);
        update(&mut a, b, c, d, x[9], 5, 0x21e1cde6, g);
        update(&mut d, a, b, c, x[14], 9, 0xc33707d6, g);
        update(&mut c, d, a, b, x[3], 14, 0xf4d50d87, g);
        update(&mut b, c, d, a, x[8], 20, 0x455a14ed, g);
        update(&mut a, b, c, d, x[13], 5, 0xa9e3e905, g);
        update(&mut d, a, b, c, x[2], 9, 0xfcefa3f8, g);
        update(&mut c, d, a, b, x[7], 14, 0x676f02d9, g);
        update(&mut b, c, d, a, x[12], 20, 0x8d2a4c8a, g);

        update(&mut a, b, c, d, x[5], 4, 0xfffa3942, h);
        update(&mut d, a, b, c, x[8], 11, 0x8771f681, h);
        update(&mut c, d, a, b, x[11], 16, 0x6d9d6122, h);
        update(&mut b, c, d, a, x[14], 23, 0xfde5380c, h);
        update(&mut a, b, c, d, x[1], 4, 0xa4beea44, h);
        update(&mut d, a, b, c, x[4], 11, 0x4bdecfa9, h);
        update(&mut c, d, a, b, x[7], 16, 0xf6bb4b60, h);
        update(&mut b, c, d, a, x[10], 23, 0xbebfbc70, h);
        update(&mut a, b, c, d, x[13], 4, 0x289b7ec6, h);
        update(&mut d, a, b, c, x[0], 11, 0xeaa127fa, h);
        update(&mut c, d, a, b, x[3], 16, 0xd4ef3085, h);
        update(&mut b, c, d, a, x[6], 23, 0x4881d05, h);
        update(&mut a, b, c, d, x[9], 4, 0xd9d4d039, h);
        update(&mut d, a, b, c, x[12], 11, 0xe6db99e5, h);
        update(&mut c, d, a, b, x[15], 16, 0x1fa27cf8, h);
        update(&mut b, c, d, a, x[2], 23, 0xc4ac5665, h);

        update(&mut a, b, c, d, x[0], 6, 0xf4292244, i);
        update(&mut d, a, b, c, x[7], 10, 0x432aff97, i);
        update(&mut c, d, a, b, x[14], 15, 0xab9423a7, i);
        update(&mut b, c, d, a, x[5], 21, 0xfc93a039, i);
        update(&mut a, b, c, d, x[12], 6, 0x655b59c3, i);
        update(&mut d, a, b, c, x[3], 10, 0x8f0ccc92, i);
        update(&mut c, d, a, b, x[10], 15, 0xffeff47d, i);
        update(&mut b, c, d, a, x[1], 21, 0x85845dd1, i);
        update(&mut a, b, c, d, x[8], 6, 0x6fa87e4f, i);
        update(&mut d, a, b, c, x[15], 10, 0xfe2ce6e0, i);
        update(&mut c, d, a, b, x[6], 15, 0xa3014314, i);
        update(&mut b, c, d, a, x[13], 21, 0x4e0811a1, i);
        update(&mut a, b, c, d, x[4], 6, 0xf7537e82, i);
        update(&mut d, a, b, c, x[11], 10, 0xbd3af235, i);
        update(&mut c, d, a, b, x[2], 15, 0x2ad7d2bb, i);
        update(&mut b, c, d, a, x[9], 21, 0xeb86d391, i);

        A = A.wrapping_add(a);
        B = B.wrapping_add(b);
        C = C.wrapping_add(c);
        D = D.wrapping_add(d);
    }

    let mut builder = String::new();
    for byte in A.to_le_bytes() {
        builder.push_str(&format!("{:02x}", byte));
    }
    for byte in B.to_le_bytes() {
        builder.push_str(&format!("{:02x}", byte));
    }
    for byte in C.to_le_bytes() {
        builder.push_str(&format!("{:02x}", byte));
    }
    for byte in D.to_le_bytes() {
        builder.push_str(&format!("{:02x}", byte));
    }

    builder
}

#[test]
fn test_process() {
    let expected = "d41d8cd98f00b204e9800998ecf8427e";
    let output = process("".to_string());
    assert_eq!(expected, output);
}
