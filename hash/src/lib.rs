#![no_std]

use alloc::{format, string::String, vec::Vec};
extern crate alloc;

pub mod md5;
pub mod sha256;
pub mod sha512;

#[inline(always)]
pub fn digest_to_hex_string(dgst: &[u8]) -> String {
    let str_vec: Vec<String> = dgst.iter().map(|b| format!("{:02x}", b)).collect();
    str_vec.join("")
}
