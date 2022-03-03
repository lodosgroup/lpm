const BLOCK_SIZE: usize = 64;
const RESULT_SIZE: usize = 32;
const STATE_SIZE: usize = 8;
const INIT_STATE: [u32; STATE_SIZE] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];
const K256: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

const fn sha256_transform(
    state: [u32; STATE_SIZE],
    cursor: usize,
    input: &[u8],
) -> [u32; STATE_SIZE] {
    let mut x = [
        u32::from_be_bytes([
            input[cursor],
            input[cursor + 1],
            input[cursor + 2],
            input[cursor + 3],
        ]),
        u32::from_be_bytes([
            input[cursor + 4],
            input[cursor + 5],
            input[cursor + 6],
            input[cursor + 7],
        ]),
        u32::from_be_bytes([
            input[cursor + 8],
            input[cursor + 9],
            input[cursor + 10],
            input[cursor + 11],
        ]),
        u32::from_be_bytes([
            input[cursor + 12],
            input[cursor + 13],
            input[cursor + 14],
            input[cursor + 15],
        ]),
        u32::from_be_bytes([
            input[cursor + 16],
            input[cursor + 17],
            input[cursor + 18],
            input[cursor + 19],
        ]),
        u32::from_be_bytes([
            input[cursor + 20],
            input[cursor + 21],
            input[cursor + 22],
            input[cursor + 23],
        ]),
        u32::from_be_bytes([
            input[cursor + 24],
            input[cursor + 25],
            input[cursor + 26],
            input[cursor + 27],
        ]),
        u32::from_be_bytes([
            input[cursor + 28],
            input[cursor + 29],
            input[cursor + 30],
            input[cursor + 31],
        ]),
        u32::from_be_bytes([
            input[cursor + 32],
            input[cursor + 33],
            input[cursor + 34],
            input[cursor + 35],
        ]),
        u32::from_be_bytes([
            input[cursor + 36],
            input[cursor + 37],
            input[cursor + 38],
            input[cursor + 39],
        ]),
        u32::from_be_bytes([
            input[cursor + 40],
            input[cursor + 41],
            input[cursor + 42],
            input[cursor + 43],
        ]),
        u32::from_be_bytes([
            input[cursor + 44],
            input[cursor + 45],
            input[cursor + 46],
            input[cursor + 47],
        ]),
        u32::from_be_bytes([
            input[cursor + 48],
            input[cursor + 49],
            input[cursor + 50],
            input[cursor + 51],
        ]),
        u32::from_be_bytes([
            input[cursor + 52],
            input[cursor + 53],
            input[cursor + 54],
            input[cursor + 55],
        ]),
        u32::from_be_bytes([
            input[cursor + 56],
            input[cursor + 57],
            input[cursor + 58],
            input[cursor + 59],
        ]),
        u32::from_be_bytes([
            input[cursor + 60],
            input[cursor + 61],
            input[cursor + 62],
            input[cursor + 63],
        ]),
    ];

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
            (($in.rotate_right(9) ^ $in).rotate_right(11) ^ $in).rotate_right(2)
        };
    }

    macro_rules! S1 {
        ($in:expr) => {
            (($in.rotate_right(14) ^ $in).rotate_right(5) ^ $in).rotate_right(6)
        };
    }

    macro_rules! s0 {
        ($in:expr) => {
            ($in.rotate_right(11) ^ $in).rotate_right(7) ^ $in.wrapping_shr(3)
        };
    }

    macro_rules! s1 {
        ($in:expr) => {
            ($in.rotate_right(2) ^ $in).rotate_right(17) ^ $in.wrapping_shr(10)
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

    macro_rules! R0 {
        ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr, $g:expr, $h:expr, $i:expr) => {
            $h = $h.wrapping_add(
                S1!($e)
                    .wrapping_add(Ch!($e, $f, $g))
                    .wrapping_add(K256[$i])
                    .wrapping_add(x[$i]),
            );
            $d = $d.wrapping_add($h);
            $h = $h.wrapping_add(S0!($a).wrapping_add(Ma!($a, $b, $c)));
        };
    }

    macro_rules! R1 {
        ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr, $g:expr, $h:expr, $i:expr, $j:expr) => {
            let i = $i as usize;
            $h = $h.wrapping_add(S1!($e).wrapping_add(Ch!($e, $f, $g)).wrapping_add(K256[i + $j]).wrapping_add({
                    x[$i] = x[$i].wrapping_add(s1!(x[i.wrapping_sub(2) & 15]).wrapping_add(x[i.wrapping_sub(7) & 15])
                                                                              .wrapping_add(s0!(x[i.wrapping_sub(15) & 15]))
                    );
                    x[$i]
                }
            ));
            $d = $d.wrapping_add($h);
            $h = $h.wrapping_add(S0!($a).wrapping_add(Ma!($a, $b, $c)));
        }
    }

    R0!(a, b, c, d, e, f, g, h, 0);
    R0!(h, a, b, c, d, e, f, g, 1);
    R0!(g, h, a, b, c, d, e, f, 2);
    R0!(f, g, h, a, b, c, d, e, 3);
    R0!(e, f, g, h, a, b, c, d, 4);
    R0!(d, e, f, g, h, a, b, c, 5);
    R0!(c, d, e, f, g, h, a, b, 6);
    R0!(b, c, d, e, f, g, h, a, 7);
    R0!(a, b, c, d, e, f, g, h, 8);
    R0!(h, a, b, c, d, e, f, g, 9);
    R0!(g, h, a, b, c, d, e, f, 10);
    R0!(f, g, h, a, b, c, d, e, 11);
    R0!(e, f, g, h, a, b, c, d, 12);
    R0!(d, e, f, g, h, a, b, c, 13);
    R0!(c, d, e, f, g, h, a, b, 14);
    R0!(b, c, d, e, f, g, h, a, 15);

    R1!(a, b, c, d, e, f, g, h, 0, 16);
    R1!(h, a, b, c, d, e, f, g, 1, 16);
    R1!(g, h, a, b, c, d, e, f, 2, 16);
    R1!(f, g, h, a, b, c, d, e, 3, 16);
    R1!(e, f, g, h, a, b, c, d, 4, 16);
    R1!(d, e, f, g, h, a, b, c, 5, 16);
    R1!(c, d, e, f, g, h, a, b, 6, 16);
    R1!(b, c, d, e, f, g, h, a, 7, 16);
    R1!(a, b, c, d, e, f, g, h, 8, 16);
    R1!(h, a, b, c, d, e, f, g, 9, 16);
    R1!(g, h, a, b, c, d, e, f, 10, 16);
    R1!(f, g, h, a, b, c, d, e, 11, 16);
    R1!(e, f, g, h, a, b, c, d, 12, 16);
    R1!(d, e, f, g, h, a, b, c, 13, 16);
    R1!(c, d, e, f, g, h, a, b, 14, 16);
    R1!(b, c, d, e, f, g, h, a, 15, 16);

    R1!(a, b, c, d, e, f, g, h, 0, 32);
    R1!(h, a, b, c, d, e, f, g, 1, 32);
    R1!(g, h, a, b, c, d, e, f, 2, 32);
    R1!(f, g, h, a, b, c, d, e, 3, 32);
    R1!(e, f, g, h, a, b, c, d, 4, 32);
    R1!(d, e, f, g, h, a, b, c, 5, 32);
    R1!(c, d, e, f, g, h, a, b, 6, 32);
    R1!(b, c, d, e, f, g, h, a, 7, 32);
    R1!(a, b, c, d, e, f, g, h, 8, 32);
    R1!(h, a, b, c, d, e, f, g, 9, 32);
    R1!(g, h, a, b, c, d, e, f, 10, 32);
    R1!(f, g, h, a, b, c, d, e, 11, 32);
    R1!(e, f, g, h, a, b, c, d, 12, 32);
    R1!(d, e, f, g, h, a, b, c, 13, 32);
    R1!(c, d, e, f, g, h, a, b, 14, 32);
    R1!(b, c, d, e, f, g, h, a, 15, 32);

    R1!(a, b, c, d, e, f, g, h, 0, 48);
    R1!(h, a, b, c, d, e, f, g, 1, 48);
    R1!(g, h, a, b, c, d, e, f, 2, 48);
    R1!(f, g, h, a, b, c, d, e, 3, 48);
    R1!(e, f, g, h, a, b, c, d, 4, 48);
    R1!(d, e, f, g, h, a, b, c, 5, 48);
    R1!(c, d, e, f, g, h, a, b, 6, 48);
    R1!(b, c, d, e, f, g, h, a, 7, 48);
    R1!(a, b, c, d, e, f, g, h, 8, 48);
    R1!(h, a, b, c, d, e, f, g, 9, 48);
    R1!(g, h, a, b, c, d, e, f, 10, 48);
    R1!(f, g, h, a, b, c, d, e, 11, 48);
    R1!(e, f, g, h, a, b, c, d, 12, 48);
    R1!(d, e, f, g, h, a, b, c, 13, 48);
    R1!(c, d, e, f, g, h, a, b, 14, 48);
    R1!(b, c, d, e, f, g, h, a, 15, 48);

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

pub const fn sha256(input: &[u8]) -> [u8; RESULT_SIZE] {
    let mut state = INIT_STATE;
    let mut cursor = 0;

    while cursor + BLOCK_SIZE <= input.len() {
        state = sha256_transform(state, cursor, input);
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

    while pos != (BLOCK_SIZE - core::mem::size_of::<u64>()) {
        pos &= BLOCK_SIZE - 1;

        if pos == 0 {
            state = sha256_transform(state, 0, &buffer);
        }

        buffer[pos] = 0;
        pos += 1;
    }

    let len = (input.len() as u64).wrapping_shl(3).to_be_bytes();
    buffer[pos] = len[0];
    buffer[pos + 1] = len[1];
    buffer[pos + 2] = len[2];
    buffer[pos + 3] = len[3];
    buffer[pos + 4] = len[4];
    buffer[pos + 5] = len[5];
    buffer[pos + 6] = len[6];
    buffer[pos + 7] = len[7];

    state = sha256_transform(state, 0, &buffer);

    let a = state[0].to_be_bytes();
    let b = state[1].to_be_bytes();
    let c = state[2].to_be_bytes();
    let d = state[3].to_be_bytes();
    let e = state[4].to_be_bytes();
    let f = state[5].to_be_bytes();
    let g = state[6].to_be_bytes();
    let h = state[7].to_be_bytes();
    [
        a[0], a[1], a[2], a[3], b[0], b[1], b[2], b[3], c[0], c[1], c[2], c[3], d[0], d[1], d[2],
        d[3], e[0], e[1], e[2], e[3], f[0], f[1], f[2], f[3], g[0], g[1], g[2], g[3], h[0], h[1],
        h[2], h[3],
    ]
}

pub struct Sha256 {
    state: [u32; STATE_SIZE],
    len: u64,
    buffer: [u8; BLOCK_SIZE],
}

impl Sha256 {
    pub const fn new() -> Self {
        Self {
            state: INIT_STATE,
            len: 0,
            buffer: [0; BLOCK_SIZE],
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub const fn const_update(mut self, input: &[u8]) -> Self {
        let num = (self.len & (BLOCK_SIZE as u64 - 1)) as usize;
        self.len += input.len() as u64;

        let mut cursor = 0;

        if num > 0 {
            let block_num = BLOCK_SIZE - num;

            if input.len() < block_num {
                let mut idx = 0;
                while idx < input.len() {
                    self.buffer[num + idx] = input[idx];
                    idx += 1;
                }
                return self;
            }

            let mut idx = 0;
            while idx < block_num {
                self.buffer[num + idx] = input[idx];
                idx += 1;
            }
            self.state = sha256_transform(self.state, 0, &self.buffer);
            cursor += block_num
        }

        while input.len() - cursor >= BLOCK_SIZE {
            self.state = sha256_transform(self.state, cursor, input);
            cursor += BLOCK_SIZE;
        }

        let remains = input.len() - cursor;
        let mut idx = 0;
        while idx < remains {
            self.buffer[idx] = input[cursor + idx];
            idx += 1;
        }

        self
    }

    pub fn update(&mut self, input: &[u8]) {
        let mut num = (self.len & (BLOCK_SIZE as u64 - 1)) as usize;
        self.len += input.len() as u64;

        let mut cursor = 0;

        if num > 0 {
            let buffer = &mut self.buffer[num..];
            num = BLOCK_SIZE - num;

            if input.len() < num {
                buffer[..input.len()].copy_from_slice(input);
                return;
            }

            buffer.copy_from_slice(&input[..num]);
            self.state = sha256_transform(self.state, 0, &self.buffer);
            cursor += num
        }

        while input.len() - cursor >= BLOCK_SIZE {
            self.state = sha256_transform(self.state, cursor, input);
            cursor += BLOCK_SIZE;
        }

        let remains = input.len() - cursor;
        if remains > 0 {
            self.buffer[..remains].copy_from_slice(&input[cursor..]);
        }
    }

    pub const fn const_result(mut self) -> [u8; RESULT_SIZE] {
        let mut pos = (self.len & (BLOCK_SIZE as u64 - 1)) as usize;

        self.buffer[pos] = 0x80;
        pos += 1;

        while pos != (BLOCK_SIZE - core::mem::size_of::<u64>()) {
            pos &= BLOCK_SIZE - 1;

            if pos == 0 {
                self.state = sha256_transform(self.state, 0, &self.buffer);
            }

            self.buffer[pos] = 0;
            pos += 1;
        }

        let len = self.len.wrapping_shl(3).to_be_bytes();
        self.buffer[pos] = len[0];
        self.buffer[pos + 1] = len[1];
        self.buffer[pos + 2] = len[2];
        self.buffer[pos + 3] = len[3];
        self.buffer[pos + 4] = len[4];
        self.buffer[pos + 5] = len[5];
        self.buffer[pos + 6] = len[6];
        self.buffer[pos + 7] = len[7];

        self.state = sha256_transform(self.state, 0, &self.buffer);

        let a = self.state[0].to_be_bytes();
        let b = self.state[1].to_be_bytes();
        let c = self.state[2].to_be_bytes();
        let d = self.state[3].to_be_bytes();
        let e = self.state[4].to_be_bytes();
        let f = self.state[5].to_be_bytes();
        let g = self.state[6].to_be_bytes();
        let h = self.state[7].to_be_bytes();
        [
            a[0], a[1], a[2], a[3], b[0], b[1], b[2], b[3], c[0], c[1], c[2], c[3], d[0], d[1],
            d[2], d[3], e[0], e[1], e[2], e[3], f[0], f[1], f[2], f[3], g[0], g[1], g[2], g[3],
            h[0], h[1], h[2], h[3],
        ]
    }

    pub fn result(&mut self) -> [u8; RESULT_SIZE] {
        let mut pos = (self.len & (BLOCK_SIZE as u64 - 1)) as usize;

        self.buffer[pos] = 0x80;
        pos += 1;

        while pos != (BLOCK_SIZE - core::mem::size_of::<u64>()) {
            pos &= BLOCK_SIZE - 1;

            if pos == 0 {
                self.state = sha256_transform(self.state, 0, &self.buffer);
            }

            self.buffer[pos] = 0;
            pos += 1;
        }

        let len = self.len.wrapping_shl(3).to_be_bytes();
        self.buffer[pos] = len[0];
        self.buffer[pos + 1] = len[1];
        self.buffer[pos + 2] = len[2];
        self.buffer[pos + 3] = len[3];
        self.buffer[pos + 4] = len[4];
        self.buffer[pos + 5] = len[5];
        self.buffer[pos + 6] = len[6];
        self.buffer[pos + 7] = len[7];

        self.state = sha256_transform(self.state, 0, &self.buffer);

        let a = self.state[0].to_be_bytes();
        let b = self.state[1].to_be_bytes();
        let c = self.state[2].to_be_bytes();
        let d = self.state[3].to_be_bytes();
        let e = self.state[4].to_be_bytes();
        let f = self.state[5].to_be_bytes();
        let g = self.state[6].to_be_bytes();
        let h = self.state[7].to_be_bytes();
        [
            a[0], a[1], a[2], a[3], b[0], b[1], b[2], b[3], c[0], c[1], c[2], c[3], d[0], d[1],
            d[2], d[3], e[0], e[1], e[2], e[3], f[0], f[1], f[2], f[3], g[0], g[1], g[2], g[3],
            h[0], h[1], h[2], h[3],
        ]
    }
}

impl super::Digest for Sha256 {
    type OutputType = [u8; RESULT_SIZE];
    type BlockType = [u8; BLOCK_SIZE];

    #[inline(always)]
    fn new() -> Self {
        Self::new()
    }

    #[inline(always)]
    fn reset(&mut self) {
        self.reset();
    }

    #[inline(always)]
    fn update(&mut self, input: &[u8]) {
        self.update(input);
    }

    #[inline(always)]
    fn result(&mut self) -> Self::OutputType {
        self.result()
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::string::{String, ToString};

    use super::*;

    fn digest_to_hex(input: [u8; RESULT_SIZE]) -> String {
        crate::DigestFmt(input).to_string()
    }

    #[test]
    fn test_simple() {
        let tests = [
            ("", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),
            ("abc", "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"),
            ("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq", "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"),
            ("abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu", "cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1"),
        ];

        let mut hasher = Sha256::new();
        let mut chunked = Sha256::new();
        for (data, ref expected) in tests.iter() {
            let data = data.as_bytes();

            let mut chunked_const = Sha256::new();
            hasher.update(data);
            for chunk in data.chunks(25) {
                chunked.update(chunk);
                chunked_const = chunked_const.const_update(chunk);
            }

            let hash = digest_to_hex(hasher.result());
            let chunked_hash = digest_to_hex(chunked.result());
            let const_hash = digest_to_hex(super::sha256(data));
            let const_chunked_hash = digest_to_hex(chunked_const.const_result());
            let const_hash_stateful =
                digest_to_hex(Sha256::new().const_update(data).const_result());

            assert_eq!(const_hash.len(), hash.len());
            assert_eq!(hash, *expected);
            assert_eq!(const_hash, *expected);
            assert_eq!(hash, chunked_hash);
            assert_eq!(hash, const_chunked_hash);
            assert_eq!(hash, const_hash_stateful);

            hasher.reset();
            chunked.reset();
        }
    }

    #[test]
    fn test_hmac() {
        let tests: [(&'static [u8], &'static [u8], &'static str); 5] = [
            (&[0x0B; 20], b"Hi There", "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"),
            (b"Jefe", b"what do ya want for nothing?", "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"),
            (&[0xAA; 20], &[0xDD; 50], "773ea91e36800e46854db8ebd09181a72959098b3ef8c122d9635514ced565fe"),
            (&[0xAA; 131], b"Test Using Larger Than Block-Size Key - Hash Key First", "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"),
            (&[0xAA; 131], b"This is a test using a larger than block-size key and a larger than block-size data. The key needs to be hashed before being used by the HMAC algorithm.", "9b09ffa71b942fcb27635fbcd5b0e944bfdc63644f0713938a7f51535c3a35e2"),
        ];

        for (key, data, ref expected) in tests.iter() {
            let hash = crate::hmac::<Sha256>(data, key);
            let hash = digest_to_hex(hash);

            assert_eq!(hash, *expected);
        }
    }
}
