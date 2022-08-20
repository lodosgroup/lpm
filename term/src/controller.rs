use std::os::raw::{c_int, c_ulong, c_ushort};
use std::os::unix::io::RawFd;
pub const STDIN_FD: RawFd = 0;
pub const STDOUT_FD: RawFd = 1;
pub const STDERR_FD: RawFd = 2;
static TIOCGWINSZ: c_ulong = 0x5413;

extern "C" {
    fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
}

/// winsize port from C
/// ```c
/// struct winsize
/// {
///     unsigned short ws_row;
///     unsigned short ws_col;
///     unsigned short ws_xpixel;
///     unsigned short ws_ypixel;
/// };
/// ```
#[derive(Default, Debug)]
pub struct TermController {
    rows: c_ushort,
    columns: c_ushort,
    x_pixels: c_ushort,
    y_pixels: c_ushort,
}

impl TermController {
    fn execute_ioctl(&mut self) {
        for fd in [STDOUT_FD, STDIN_FD, STDERR_FD] {
            #[allow(unsafe_code)]
            unsafe {
                if ioctl(fd, TIOCGWINSZ, &mut *self) != -1 {
                    break;
                }
            }
        }
    }

    pub fn new() -> TermController {
        let mut w: TermController = Default::default();
        w.execute_ioctl();
        w
    }

    #[inline(always)]
    pub fn rows(&self) -> usize {
        self.rows as usize
    }

    #[inline(always)]
    pub fn columns(&self) -> usize {
        self.columns as usize
    }

    #[inline(always)]
    pub fn x_pixels(&self) -> usize {
        self.x_pixels as usize
    }

    #[inline(always)]
    pub fn y_pixels(&self) -> usize {
        self.y_pixels as usize
    }
}
