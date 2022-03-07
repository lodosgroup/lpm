const BLOCK_SIZE: usize = 128;
const RESULT_SIZE: usize = 64;
const STATE_SIZE: usize = 8;
const INIT_STATE: [u64; STATE_SIZE] = [
    0x6a09e667f3bcc908,
    0xbb67ae8584caa73b,
    0x3c6ef372fe94f82b,
    0xa54ff53a5f1d36f1,
    0x510e527fade682d1,
    0x9b05688c2b3e6c1f,
    0x1f83d9abfb41bd6b,
    0x5be0cd19137e2179,
];
const K512: [u64; 80] = [
    0x428a2f98d728ae22,
    0x7137449123ef65cd,
    0xb5c0fbcfec4d3b2f,
    0xe9b5dba58189dbbc,
    0x3956c25bf348b538,
    0x59f111f1b605d019,
    0x923f82a4af194f9b,
    0xab1c5ed5da6d8118,
    0xd807aa98a3030242,
    0x12835b0145706fbe,
    0x243185be4ee4b28c,
    0x550c7dc3d5ffb4e2,
    0x72be5d74f27b896f,
    0x80deb1fe3b1696b1,
    0x9bdc06a725c71235,
    0xc19bf174cf692694,
    0xe49b69c19ef14ad2,
    0xefbe4786384f25e3,
    0x0fc19dc68b8cd5b5,
    0x240ca1cc77ac9c65,
    0x2de92c6f592b0275,
    0x4a7484aa6ea6e483,
    0x5cb0a9dcbd41fbd4,
    0x76f988da831153b5,
    0x983e5152ee66dfab,
    0xa831c66d2db43210,
    0xb00327c898fb213f,
    0xbf597fc7beef0ee4,
    0xc6e00bf33da88fc2,
    0xd5a79147930aa725,
    0x06ca6351e003826f,
    0x142929670a0e6e70,
    0x27b70a8546d22ffc,
    0x2e1b21385c26c926,
    0x4d2c6dfc5ac42aed,
    0x53380d139d95b3df,
    0x650a73548baf63de,
    0x766a0abb3c77b2a8,
    0x81c2c92e47edaee6,
    0x92722c851482353b,
    0xa2bfe8a14cf10364,
    0xa81a664bbc423001,
    0xc24b8b70d0f89791,
    0xc76c51a30654be30,
    0xd192e819d6ef5218,
    0xd69906245565a910,
    0xf40e35855771202a,
    0x106aa07032bbd1b8,
    0x19a4c116b8d2d0c8,
    0x1e376c085141ab53,
    0x2748774cdf8eeb99,
    0x34b0bcb5e19b48a8,
    0x391c0cb3c5c95a63,
    0x4ed8aa4ae3418acb,
    0x5b9cca4f7763e373,
    0x682e6ff3d6b2b8a3,
    0x748f82ee5defb2fc,
    0x78a5636f43172f60,
    0x84c87814a1f0ab72,
    0x8cc702081a6439ec,
    0x90befffa23631e28,
    0xa4506cebde82bde9,
    0xbef9a3f7b2c67915,
    0xc67178f2e372532b,
    0xca273eceea26619c,
    0xd186b8c721c0c207,
    0xeada7dd6cde0eb1e,
    0xf57d4f7fee6ed178,
    0x06f067aa72176fba,
    0x0a637dc5a2c898a6,
    0x113f9804bef90dae,
    0x1b710b35131c471b,
    0x28db77f523047d84,
    0x32caab7b40c72493,
    0x3c9ebe0a15c9bebc,
    0x431d67c49c100d4c,
    0x4cc5d4becb3e42b6,
    0x597f299cfc657e2a,
    0x5fcb6fab3ad6faec,
    0x6c44198c4a475817,
];

const fn sha512_transform(
    state: [u64; STATE_SIZE],
    cursor: usize,
    input: &[u8],
) -> [u64; STATE_SIZE] {
    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    let mut e = state[4];
    let mut f = state[5];
    let mut g = state[6];
    let mut h = state[7];

    macro_rules! S0 {
        ($in:expr) => {
            (($in.rotate_right(5) ^ $in).rotate_right(6) ^ $in).rotate_right(28)
        };
    }

    macro_rules! S1 {
        ($in:expr) => {
            (($in.rotate_right(23) ^ $in).rotate_right(4) ^ $in).rotate_right(14)
        };
    }

    macro_rules! s0 {
        ($in:expr) => {
            ($in.rotate_right(7) ^ $in).rotate_right(1) ^ $in.wrapping_shr(7)
        };
    }

    macro_rules! s1 {
        ($in:expr) => {
            ($in.rotate_right(42) ^ $in).rotate_right(19) ^ $in.wrapping_shr(6)
        };
    }

    macro_rules! Ch {
        ($x:expr, $y:expr, $z:expr) => {
            (($z) ^ (($x) & (($y) ^ ($z))))
        };
    }

    macro_rules! Ma {
        ($x:expr, $y:expr, $z:expr) => {
            ((($x) & ($y)) | (($z) & (($x) | ($y))))
        };
    }

    const fn read_u64(input: &[u8], cursor: usize) -> u64 {
        u64::from_be_bytes([
            input[cursor],
            input[cursor + 1],
            input[cursor + 2],
            input[cursor + 3],
            input[cursor + 4],
            input[cursor + 5],
            input[cursor + 6],
            input[cursor + 7],
        ])
    }

    let mut x = [0u64; 80];
    let mut idx = 0;
    while idx < 16 {
        x[idx] = read_u64(input, cursor + idx * 8);
        idx += 1;
    }

    while idx < x.len() {
        x[idx] = s1!(x[idx - 2])
            .wrapping_add(x[idx - 7])
            .wrapping_add(s0!(x[idx - 15]))
            .wrapping_add(x[idx - 16]);
        idx += 1;
    }

    macro_rules! R {
        ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr, $g:expr, $h:expr, $i:expr) => {
            $h = $h.wrapping_add(
                S1!($e)
                    .wrapping_add(Ch!($e, $f, $g))
                    .wrapping_add(K512[$i])
                    .wrapping_add(x[$i]),
            );
            $d = $d.wrapping_add($h);
            $h = $h.wrapping_add(S0!($a).wrapping_add(Ma!($a, $b, $c)));
        };
    }

    idx = 0;
    while idx < 80 {
        R!(a, b, c, d, e, f, g, h, idx);
        R!(h, a, b, c, d, e, f, g, idx + 1);
        R!(g, h, a, b, c, d, e, f, idx + 2);
        R!(f, g, h, a, b, c, d, e, idx + 3);
        R!(e, f, g, h, a, b, c, d, idx + 4);
        R!(d, e, f, g, h, a, b, c, idx + 5);
        R!(c, d, e, f, g, h, a, b, idx + 6);
        R!(b, c, d, e, f, g, h, a, idx + 7);

        idx += 8;
    }

    [
        state[0].wrapping_add(a),
        state[1].wrapping_add(b),
        state[2].wrapping_add(c),
        state[3].wrapping_add(d),
        state[4].wrapping_add(e),
        state[5].wrapping_add(f),
        state[6].wrapping_add(g),
        state[7].wrapping_add(h),
    ]
}

pub const fn digest(input: &[u8]) -> [u8; RESULT_SIZE] {
    let mut state = INIT_STATE;
    let mut cursor = 0;

    while cursor + BLOCK_SIZE <= input.len() {
        state = sha512_transform(state, cursor, input);
        cursor += BLOCK_SIZE;
    }

    let mut pos = 0;
    let mut buffer = [0; BLOCK_SIZE];

    while pos < input.len() - cursor {
        buffer[pos] = input[cursor + pos];
        pos += 1;
    }
    buffer[pos] = 0x80;
    pos += 1;

    while pos != (BLOCK_SIZE - (2 * core::mem::size_of::<u64>())) {
        pos &= BLOCK_SIZE - 1;

        if pos == 0 {
            state = sha512_transform(state, 0, &buffer);
        }

        buffer[pos] = 0;
        pos += 1;
    }

    let input_len = input.len() as u64;
    let len_lo = input_len.wrapping_shl(3).to_be_bytes();
    let len_hi = input_len.wrapping_shr(64 - 3).to_be_bytes();
    buffer[pos] = len_hi[0];
    buffer[pos + 1] = len_hi[1];
    buffer[pos + 2] = len_hi[2];
    buffer[pos + 3] = len_hi[3];
    buffer[pos + 4] = len_hi[4];
    buffer[pos + 5] = len_hi[5];
    buffer[pos + 6] = len_hi[6];
    buffer[pos + 7] = len_hi[7];

    buffer[pos + 8] = len_lo[0];
    buffer[pos + 9] = len_lo[1];
    buffer[pos + 10] = len_lo[2];
    buffer[pos + 11] = len_lo[3];
    buffer[pos + 12] = len_lo[4];
    buffer[pos + 13] = len_lo[5];
    buffer[pos + 14] = len_lo[6];
    buffer[pos + 15] = len_lo[7];

    state = sha512_transform(state, 0, &buffer);

    let a = state[0].to_be_bytes();
    let b = state[1].to_be_bytes();
    let c = state[2].to_be_bytes();
    let d = state[3].to_be_bytes();
    let e = state[4].to_be_bytes();
    let f = state[5].to_be_bytes();
    let g = state[6].to_be_bytes();
    let h = state[7].to_be_bytes();
    [
        a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7], b[0], b[1], b[2], b[3], b[4], b[5], b[6],
        b[7], c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7], d[0], d[1], d[2], d[3], d[4], d[5],
        d[6], d[7], e[0], e[1], e[2], e[3], e[4], e[5], e[6], e[7], f[0], f[1], f[2], f[3], f[4],
        f[5], f[6], f[7], g[0], g[1], g[2], g[3], g[4], g[5], g[6], g[7], h[0], h[1], h[2], h[3],
        h[4], h[5], h[6], h[7],
    ]
}

#[cfg(test)]
mod tests {
    use super::digest;
    use crate::digest_to_hex_string;

    #[test]
    fn test_digest_and_hex() {
        let x = b"";
        let x_byte_array = [
            207, 131, 225, 53, 126, 239, 184, 189, 241, 84, 40, 80, 214, 109, 128, 7, 214, 32, 228,
            5, 11, 87, 21, 220, 131, 244, 169, 33, 211, 108, 233, 206, 71, 208, 209, 60, 93, 133,
            242, 176, 255, 131, 24, 210, 135, 126, 236, 47, 99, 185, 49, 189, 71, 65, 122, 129,
            165, 56, 50, 122, 249, 39, 218, 62,
        ];
        let x_sha512_str = String::from("cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e");

        let y = b"Onur Ozkan - LodPM Core Developer & Maintainer";
        let y_byte_array = [
            243, 222, 25, 66, 101, 122, 84, 22, 37, 46, 150, 46, 175, 193, 52, 177, 234, 0, 199,
            173, 16, 190, 9, 150, 163, 254, 32, 37, 75, 127, 144, 254, 194, 165, 227, 126, 82, 209,
            24, 67, 199, 80, 189, 176, 157, 143, 190, 93, 48, 249, 202, 94, 213, 25, 9, 19, 237,
            162, 78, 213, 204, 42, 252, 48,
        ];
        let y_sha512_str = String::from("f3de1942657a5416252e962eafc134b1ea00c7ad10be0996a3fe20254b7f90fec2a5e37e52d11843c750bdb09d8fbe5d30f9ca5ed5190913eda24ed5cc2afc30");

        let z = b"Kebab is the best food!!1";
        let z_byte_array = [
            175, 232, 62, 23, 105, 211, 28, 88, 13, 166, 232, 197, 245, 227, 92, 219, 105, 31, 33,
            83, 255, 83, 18, 64, 184, 47, 119, 170, 206, 31, 242, 231, 124, 169, 154, 181, 90, 69,
            49, 224, 131, 40, 120, 211, 144, 11, 241, 45, 29, 17, 82, 168, 115, 227, 247, 53, 224,
            31, 132, 99, 138, 178, 101, 222,
        ];
        let z_sha512_str = String::from("afe83e1769d31c580da6e8c5f5e35cdb691f2153ff531240b82f77aace1ff2e77ca99ab55a4531e0832878d3900bf12d1d1152a873e3f735e01f84638ab265de");

        let t = b"coulda, woulda, shoulda";
        let t_byte_array = [
            101, 184, 191, 20, 176, 17, 10, 80, 13, 233, 125, 122, 177, 187, 233, 218, 137, 220,
            100, 10, 182, 25, 143, 222, 98, 226, 18, 111, 193, 1, 99, 114, 53, 123, 123, 227, 162,
            23, 227, 155, 123, 68, 212, 244, 123, 167, 212, 213, 37, 206, 15, 100, 48, 129, 120,
            23, 84, 122, 217, 120, 155, 144, 220, 189,
        ];
        let t_sha512_str = String::from("65b8bf14b0110a500de97d7ab1bbe9da89dc640ab6198fde62e2126fc1016372357b7be3a217e39b7b44d4f47ba7d4d525ce0f6430817817547ad9789b90dcbd");

        assert!(digest(x) == x_byte_array);
        assert!(digest_to_hex_string(&digest(x)) == x_sha512_str);

        assert!(digest(y) == y_byte_array);
        assert!(digest_to_hex_string(&digest(y)) == y_sha512_str);

        assert!(digest(z) == z_byte_array);
        assert!(digest_to_hex_string(&digest(z)) == z_sha512_str);

        assert!(digest(t) == t_byte_array);
        assert!(digest_to_hex_string(&digest(t)) == t_sha512_str);
    }
}
