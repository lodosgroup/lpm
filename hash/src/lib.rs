pub trait Digest {
    type OutputType: AsRef<[u8]> + AsMut<[u8]> + Copy;
    type BlockType: AsRef<[u8]> + AsMut<[u8]> + Copy;

    fn new() -> Self;
    fn reset(&mut self);
    fn update(&mut self, input: &[u8]);
    fn result(&mut self) -> Self::OutputType;
}

pub struct HmacKey<D: Digest> {
    key: D::BlockType,
}

impl<D: Digest> HmacKey<D> {
    pub fn new(secret: &[u8]) -> Self {
        let mut inner: D::BlockType = unsafe {
            core::mem::MaybeUninit::zeroed().assume_init()
        };
        let key = inner.as_mut();

        if secret.len() <= key.len() {
            key[..secret.len()].copy_from_slice(secret);
        } else {
        let mut algo = D::new();
            algo.update(secret);
            let hash = algo.result();
            let hash = hash.as_ref();
            key[..hash.len()].copy_from_slice(hash);
            algo.reset();
        }

        for byte in key.iter_mut() {
            *byte ^= 0x36;
        }

        Self {
            key: inner,
        }
    }

    pub fn sign(&self, input: &[u8]) -> D::OutputType {
        let mut key = self.key;
        let key = key.as_mut();

        let mut algo = D::new();
        algo.update(key);
        algo.update(input);
        let inner_result = algo.result();
        algo.reset();

        for byte in key.iter_mut() {
            *byte ^= 0x36 ^ 0x5C;
        }
        algo.update(key);
        algo.update(inner_result.as_ref());
        algo.result()
    }
}

pub fn hmac<D: Digest>(input: &[u8], secret: &[u8]) -> D::OutputType {
    let key = HmacKey::<D>::new(secret);
    key.sign(input)
}

pub struct DigestFmt<T>(pub T);

impl<T: AsRef<[u8]>> core::fmt::Display for DigestFmt<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for byt in self.0.as_ref() {
            f.write_fmt(format_args!("{:02x}", byt))?;
        }
        Ok(())
    }
}

mod md5;
pub use md5::{md5, Md5};

mod sha256;
pub use sha256::{sha256, Sha256};

mod sha512;
pub use sha512::{sha512, Sha512};
