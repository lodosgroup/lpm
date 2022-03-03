const BLOCK_SIZE: usize = 64;
const STATE_SIZE: usize = 4;
const RESULT_SIZE: usize = 16;
const INIT_STATE: [u32; STATE_SIZE] = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476];

const S11: u32 = 7;
const S12: u32 = 12;
const S13: u32 = 17;
const S14: u32 = 22;
const S21: u32 = 5;
const S22: u32 = 9;
const S23: u32 = 14;
const S24: u32 = 20;
const S31: u32 = 4;
const S32: u32 = 11;
const S33: u32 = 16;
const S34: u32 = 23;
const S41: u32 = 6;
const S42: u32 = 10;
const S43: u32 = 15;
const S44: u32 = 21;

#[inline(always)]
const fn f(x: u32, y: u32, z: u32) -> u32 {
    (x & y) | (!x & z)
}

#[inline(always)]
const fn g(x: u32, y: u32, z: u32) -> u32 {
    (x & z) | (y & !z)
}

#[inline(always)]
const fn h(x: u32, y: u32, z: u32) -> u32 {
    x ^ y ^ z
}

#[inline(always)]
const fn i(x: u32, y: u32, z: u32) -> u32 {
    y ^ (x | !z)
}

macro_rules! FF {
    ($a:expr, $b:expr, $c:expr, $d:expr, $x:expr, $s:expr, $ac:expr) => {
        $a = $a.wrapping_add(f($b, $c, $d).wrapping_add($x).wrapping_add($ac));
        $a = $a.rotate_left($s);
        $a = $a.wrapping_add($b);
    };
}

macro_rules! GG {
    ($a:expr, $b:expr, $c:expr, $d:expr, $x:expr, $s:expr, $ac:expr) => {
        $a = $a.wrapping_add(g($b, $c, $d).wrapping_add($x).wrapping_add($ac));
        $a = $a.rotate_left($s);
        $a = $a.wrapping_add($b);
    };
}

macro_rules! HH {
    ($a:expr, $b:expr, $c:expr, $d:expr, $x:expr, $s:expr, $ac:expr) => {
        $a = $a.wrapping_add(h($b, $c, $d).wrapping_add($x).wrapping_add($ac));
        $a = $a.rotate_left($s);
        $a = $a.wrapping_add($b);
    };
}

macro_rules! II {
    ($a:expr, $b:expr, $c:expr, $d:expr, $x:expr, $s:expr, $ac:expr) => {
        $a = $a.wrapping_add(i($b, $c, $d).wrapping_add($x).wrapping_add($ac));
        $a = $a.rotate_left($s);
        $a = $a.wrapping_add($b);
    };
}

const fn md5_transform(
    mut state: [u32; STATE_SIZE],
    cursor: usize,
    input: &[u8],
) -> [u32; STATE_SIZE] {
    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];

    let x = [
        u32::from_le_bytes([
            input[cursor],
            input[cursor + 1],
            input[cursor + 2],
            input[cursor + 3],
        ]),
        u32::from_le_bytes([
            input[cursor + 4],
            input[cursor + 5],
            input[cursor + 6],
            input[cursor + 7],
        ]),
        u32::from_le_bytes([
            input[cursor + 8],
            input[cursor + 9],
            input[cursor + 10],
            input[cursor + 11],
        ]),
        u32::from_le_bytes([
            input[cursor + 12],
            input[cursor + 13],
            input[cursor + 14],
            input[cursor + 15],
        ]),
        u32::from_le_bytes([
            input[cursor + 16],
            input[cursor + 17],
            input[cursor + 18],
            input[cursor + 19],
        ]),
        u32::from_le_bytes([
            input[cursor + 20],
            input[cursor + 21],
            input[cursor + 22],
            input[cursor + 23],
        ]),
        u32::from_le_bytes([
            input[cursor + 24],
            input[cursor + 25],
            input[cursor + 26],
            input[cursor + 27],
        ]),
        u32::from_le_bytes([
            input[cursor + 28],
            input[cursor + 29],
            input[cursor + 30],
            input[cursor + 31],
        ]),
        u32::from_le_bytes([
            input[cursor + 32],
            input[cursor + 33],
            input[cursor + 34],
            input[cursor + 35],
        ]),
        u32::from_le_bytes([
            input[cursor + 36],
            input[cursor + 37],
            input[cursor + 38],
            input[cursor + 39],
        ]),
        u32::from_le_bytes([
            input[cursor + 40],
            input[cursor + 41],
            input[cursor + 42],
            input[cursor + 43],
        ]),
        u32::from_le_bytes([
            input[cursor + 44],
            input[cursor + 45],
            input[cursor + 46],
            input[cursor + 47],
        ]),
        u32::from_le_bytes([
            input[cursor + 48],
            input[cursor + 49],
            input[cursor + 50],
            input[cursor + 51],
        ]),
        u32::from_le_bytes([
            input[cursor + 52],
            input[cursor + 53],
            input[cursor + 54],
            input[cursor + 55],
        ]),
        u32::from_le_bytes([
            input[cursor + 56],
            input[cursor + 57],
            input[cursor + 58],
            input[cursor + 59],
        ]),
        u32::from_le_bytes([
            input[cursor + 60],
            input[cursor + 61],
            input[cursor + 62],
            input[cursor + 63],
        ]),
    ];

    FF!(a, b, c, d, x[0], S11, 0xd76aa478);
    FF!(d, a, b, c, x[1], S12, 0xe8c7b756);
    FF!(c, d, a, b, x[2], S13, 0x242070db);
    FF!(b, c, d, a, x[3], S14, 0xc1bdceee);
    FF!(a, b, c, d, x[4], S11, 0xf57c0faf);
    FF!(d, a, b, c, x[5], S12, 0x4787c62a);
    FF!(c, d, a, b, x[6], S13, 0xa8304613);
    FF!(b, c, d, a, x[7], S14, 0xfd469501);
    FF!(a, b, c, d, x[8], S11, 0x698098d8);
    FF!(d, a, b, c, x[9], S12, 0x8b44f7af);
    FF!(c, d, a, b, x[10], S13, 0xffff5bb1);
    FF!(b, c, d, a, x[11], S14, 0x895cd7be);
    FF!(a, b, c, d, x[12], S11, 0x6b901122);
    FF!(d, a, b, c, x[13], S12, 0xfd987193);
    FF!(c, d, a, b, x[14], S13, 0xa679438e);
    FF!(b, c, d, a, x[15], S14, 0x49b40821);

    GG!(a, b, c, d, x[1], S21, 0xf61e2562);
    GG!(d, a, b, c, x[6], S22, 0xc040b340);
    GG!(c, d, a, b, x[11], S23, 0x265e5a51);
    GG!(b, c, d, a, x[0], S24, 0xe9b6c7aa);
    GG!(a, b, c, d, x[5], S21, 0xd62f105d);
    GG!(d, a, b, c, x[10], S22, 0x2441453);
    GG!(c, d, a, b, x[15], S23, 0xd8a1e681);
    GG!(b, c, d, a, x[4], S24, 0xe7d3fbc8);
    GG!(a, b, c, d, x[9], S21, 0x21e1cde6);
    GG!(d, a, b, c, x[14], S22, 0xc33707d6);
    GG!(c, d, a, b, x[3], S23, 0xf4d50d87);
    GG!(b, c, d, a, x[8], S24, 0x455a14ed);
    GG!(a, b, c, d, x[13], S21, 0xa9e3e905);
    GG!(d, a, b, c, x[2], S22, 0xfcefa3f8);
    GG!(c, d, a, b, x[7], S23, 0x676f02d9);
    GG!(b, c, d, a, x[12], S24, 0x8d2a4c8a);

    HH!(a, b, c, d, x[5], S31, 0xfffa3942);
    HH!(d, a, b, c, x[8], S32, 0x8771f681);
    HH!(c, d, a, b, x[11], S33, 0x6d9d6122);
    HH!(b, c, d, a, x[14], S34, 0xfde5380c);
    HH!(a, b, c, d, x[1], S31, 0xa4beea44);
    HH!(d, a, b, c, x[4], S32, 0x4bdecfa9);
    HH!(c, d, a, b, x[7], S33, 0xf6bb4b60);
    HH!(b, c, d, a, x[10], S34, 0xbebfbc70);
    HH!(a, b, c, d, x[13], S31, 0x289b7ec6);
    HH!(d, a, b, c, x[0], S32, 0xeaa127fa);
    HH!(c, d, a, b, x[3], S33, 0xd4ef3085);
    HH!(b, c, d, a, x[6], S34, 0x4881d05);
    HH!(a, b, c, d, x[9], S31, 0xd9d4d039);
    HH!(d, a, b, c, x[12], S32, 0xe6db99e5);
    HH!(c, d, a, b, x[15], S33, 0x1fa27cf8);
    HH!(b, c, d, a, x[2], S34, 0xc4ac5665);

    II!(a, b, c, d, x[0], S41, 0xf4292244);
    II!(d, a, b, c, x[7], S42, 0x432aff97);
    II!(c, d, a, b, x[14], S43, 0xab9423a7);
    II!(b, c, d, a, x[5], S44, 0xfc93a039);
    II!(a, b, c, d, x[12], S41, 0x655b59c3);
    II!(d, a, b, c, x[3], S42, 0x8f0ccc92);
    II!(c, d, a, b, x[10], S43, 0xffeff47d);
    II!(b, c, d, a, x[1], S44, 0x85845dd1);
    II!(a, b, c, d, x[8], S41, 0x6fa87e4f);
    II!(d, a, b, c, x[15], S42, 0xfe2ce6e0);
    II!(c, d, a, b, x[6], S43, 0xa3014314);
    II!(b, c, d, a, x[13], S44, 0x4e0811a1);
    II!(a, b, c, d, x[4], S41, 0xf7537e82);
    II!(d, a, b, c, x[11], S42, 0xbd3af235);
    II!(c, d, a, b, x[2], S43, 0x2ad7d2bb);
    II!(b, c, d, a, x[9], S44, 0xeb86d391);

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);

    state
}

pub const fn md5(input: &[u8]) -> [u8; 16] {
    let mut state = INIT_STATE;
    let mut cursor = 0;

    while cursor + 64 <= input.len() {
        state = md5_transform(state, cursor, input);
        cursor += 64;
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
            state = md5_transform(state, 0, &buffer);
        }

        buffer[pos] = 0;
        pos += 1;
    }

    let len = (input.len() as u64).wrapping_shl(3).to_le_bytes();
    buffer[pos] = len[0];
    buffer[pos + 1] = len[1];
    buffer[pos + 2] = len[2];
    buffer[pos + 3] = len[3];
    buffer[pos + 4] = len[4];
    buffer[pos + 5] = len[5];
    buffer[pos + 6] = len[6];
    buffer[pos + 7] = len[7];

    state = md5_transform(state, 0, &buffer);

    let a = state[0].to_le_bytes();
    let b = state[1].to_le_bytes();
    let c = state[2].to_le_bytes();
    let d = state[3].to_le_bytes();
    [
        a[0], a[1], a[2], a[3], b[0], b[1], b[2], b[3], c[0], c[1], c[2], c[3], d[0], d[1], d[2],
        d[3],
    ]
}

pub struct Md5 {
    state: [u32; STATE_SIZE],
    len: u64,
    buffer: [u8; BLOCK_SIZE],
}

impl Md5 {
    const RESULT_SIZE: usize = RESULT_SIZE;
    const BLOCK_SIZE: usize = BLOCK_SIZE;

    pub const fn new() -> Self {
        Self {
            state: INIT_STATE,
            len: 0,
            buffer: [0; Self::BLOCK_SIZE],
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
            self.state = md5_transform(self.state, 0, &self.buffer);
            cursor += block_num
        }

        while input.len() - cursor >= BLOCK_SIZE {
            self.state = md5_transform(self.state, cursor, input);
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
            self.state = md5_transform(self.state, 0, &self.buffer);
            cursor += num
        }

        while input.len() - cursor >= BLOCK_SIZE {
            self.state = md5_transform(self.state, cursor, input);
            cursor += BLOCK_SIZE;
        }

        let remains = input.len() - cursor;
        if remains > 0 {
            self.buffer[..remains].copy_from_slice(&input[cursor..]);
        }
    }

    pub const fn const_result(mut self) -> [u8; Self::RESULT_SIZE] {
        let mut pos = (self.len & (BLOCK_SIZE as u64 - 1)) as usize;

        self.buffer[pos] = 0x80;
        pos += 1;

        while pos != (BLOCK_SIZE - core::mem::size_of::<u64>()) {
            pos &= BLOCK_SIZE - 1;

            if pos == 0 {
                self.state = md5_transform(self.state, 0, &self.buffer);
            }

            self.buffer[pos] = 0;
            pos += 1;
        }

        let len = self.len.wrapping_shl(3).to_le_bytes();
        self.buffer[pos] = len[0];
        self.buffer[pos + 1] = len[1];
        self.buffer[pos + 2] = len[2];
        self.buffer[pos + 3] = len[3];
        self.buffer[pos + 4] = len[4];
        self.buffer[pos + 5] = len[5];
        self.buffer[pos + 6] = len[6];
        self.buffer[pos + 7] = len[7];

        self.state = md5_transform(self.state, 0, &self.buffer);

        let a = self.state[0].to_le_bytes();
        let b = self.state[1].to_le_bytes();
        let c = self.state[2].to_le_bytes();
        let d = self.state[3].to_le_bytes();
        [
            a[0], a[1], a[2], a[3], b[0], b[1], b[2], b[3], c[0], c[1], c[2], c[3], d[0], d[1],
            d[2], d[3],
        ]
    }

    pub fn result(&mut self) -> [u8; Self::RESULT_SIZE] {
        let mut pos = (self.len & (BLOCK_SIZE as u64 - 1)) as usize;

        self.buffer[pos] = 0x80;
        pos += 1;

        while pos != (BLOCK_SIZE - core::mem::size_of::<u64>()) {
            pos &= BLOCK_SIZE - 1;

            if pos == 0 {
                self.state = md5_transform(self.state, 0, &self.buffer);
            }

            self.buffer[pos] = 0;
            pos += 1;
        }

        let len = self.len.wrapping_shl(3).to_le_bytes();
        self.buffer[pos] = len[0];
        self.buffer[pos + 1] = len[1];
        self.buffer[pos + 2] = len[2];
        self.buffer[pos + 3] = len[3];
        self.buffer[pos + 4] = len[4];
        self.buffer[pos + 5] = len[5];
        self.buffer[pos + 6] = len[6];
        self.buffer[pos + 7] = len[7];

        self.state = md5_transform(self.state, 0, &self.buffer);

        let a = self.state[0].to_le_bytes();
        let b = self.state[1].to_le_bytes();
        let c = self.state[2].to_le_bytes();
        let d = self.state[3].to_le_bytes();
        [
            a[0], a[1], a[2], a[3], b[0], b[1], b[2], b[3], c[0], c[1], c[2], c[3], d[0], d[1],
            d[2], d[3],
        ]
    }
}

impl super::Digest for Md5 {
    type OutputType = [u8; Self::RESULT_SIZE];
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
            ("", "d41d8cd98f00b204e9800998ecf8427e"),
            ("a", "0cc175b9c0f1b6a831c399e269772661"),
            ("abc", "900150983cd24fb0d6963f7d28e17f72"),
            ("message digest", "f96b697d7cb7938d525a2f31aaf161d0"),
            (
                "abcdefghijklmnopqrstuvwxyz",
                "c3fcd3d76192e4007dfb496cca67e13b",
            ),
            (
                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
                "d174ab98d277d9f5a5611c2c9f419d9f",
            ),
            (
                "12345678901234567890123456789012345678901234567890123456789012345678901234567890",
                "57edf4a22be3c955ac49da2e2107b67a",
            ),
        ];

        let mut md5 = Md5::new();
        let mut chunked_md5 = Md5::new();
        for (data, ref expected) in tests.iter() {
            let data = data.as_bytes();

            let mut chunked_const = Md5::new();
            md5.update(data);
            for chunk in data.chunks(10) {
                chunked_md5.update(chunk);
                chunked_const = chunked_const.const_update(chunk);
            }

            let hash = digest_to_hex(md5.result());
            let const_hash = digest_to_hex(super::md5(data));
            let chunked_hash = digest_to_hex(chunked_md5.result());
            let const_chunked_hash = digest_to_hex(chunked_const.const_result());
            let const_hash_stateful = digest_to_hex(Md5::new().const_update(data).const_result());

            assert_eq!(const_hash.len(), hash.len());
            assert_eq!(hash, *expected);
            assert_eq!(hash, const_hash);
            assert_eq!(hash, chunked_hash);
            assert_eq!(hash, const_chunked_hash);
            assert_eq!(hash, const_hash_stateful);

            md5.reset();
            chunked_md5.reset();
        }
    }

    #[test]
    fn test_hmac() {
        let tests: [(&'static [u8], &'static [u8], &'static str); 8] = [
            (b"", b"", "74e6f7298a9c2d168935f58c001bad88"),
            (
                b"key",
                b"The quick brown fox jumps over the lazy dog",
                "80070713463e7749b90c2dc24911e275",
            ),
            (
                b"Jefe",
                b"what do ya want for nothing?",
                "750c783e6ab0b503eaa86e310a5db738",
            ),
            (&[0xAA; 16], &[0xDD; 50], "56be34521d144c88dbb8c733f0e8b3f6"),
            (&[0x0B; 16], b"Hi There", "9294727a3638bb1c13f48ef8158bfc9d"),
            (
                &[0x0C; 16],
                b"Test With Truncation",
                "56461ef2342edc00f9bab995690efd4c",
            ),
            (
                &[0xAA; 80],
                b"Test Using Larger Than Block-Size Key - Hash Key First",
                "6b1ab7fe4bd7bf8f0b62e6ce61b9d0cd",
            ),
            (
                &[0xAA; 80],
                b"Test Using Larger Than Block-Size Key and Larger Than One Block-Size Data",
                "6f630fad67cda0ee1fb1f562db3aa53e",
            ),
        ];

        for (key, data, ref expected) in tests.iter() {
            let hash = crate::hmac::<Md5>(data, key);
            let hash = digest_to_hex(hash);

            assert_eq!(hash.len(), hash.len());
            assert_eq!(hash, *expected);
        }
    }
}
