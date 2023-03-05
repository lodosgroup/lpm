use std::os::raw::{c_char, c_int};

#[allow(non_camel_case_types)]
pub type MODE_T = u32;

// #[allow(non_camel_case_types)]
// pub type PID_T = u32;

#[allow(non_camel_case_types)]
pub type GID_T = u32;

#[allow(non_camel_case_types)]
pub type UID_T = u32;

pub const S_IFMT: MODE_T = 61440;
pub const S_IFLNK: MODE_T = 40960;
pub const S_IFREG: MODE_T = 32768;
pub const S_IFCHR: MODE_T = 8192;
pub const S_IFBLK: MODE_T = 24576;
pub const S_IFIFO: MODE_T = 4096;
pub const S_IFDIR: MODE_T = 16384;

extern "C" {
    pub fn lchown(path: *const c_char, uid: UID_T, gid: GID_T) -> c_int;
    pub fn fchown(fd: c_int, owner: UID_T, group: GID_T) -> c_int;
}
