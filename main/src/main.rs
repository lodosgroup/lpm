#![allow(dead_code)] // for debugging

// BEGIN TERMINAL SIZE CALCULATION POC
use std::os::raw::{c_int, c_ulong, c_ushort};
use std::os::unix::io::RawFd;
pub const STDIN_FD: RawFd = 0;
pub const STDOUT_FD: RawFd = 1;
pub const STDERR_FD: RawFd = 2;
static TIOCGWINSZ: c_ulong = 0x5413;

// winsize port of C
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

    fn new() -> TermController {
        let mut w: TermController = Default::default();
        w.execute_ioctl();
        w
    }

    fn get_columns_and_rows() -> (usize, usize) {
        let mut w: TermController = Default::default();
        w.execute_ioctl();
        (w.columns as usize, w.rows as usize)
    }

    fn get_xy_pixels() -> (usize, usize) {
        let mut w: TermController = Default::default();
        w.execute_ioctl();
        (w.x_pixels as usize, w.y_pixels as usize)
    }
}

extern "C" {
    fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
}

// END TERMINAL SIZE CALCULATION POC

use std::{
    io::{self, Stderr, Stdout, Write},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

struct ProgressBar<'a> {
    init_message: &'a str,
    final_message: &'a str,
    states: Vec<ProgressState>,
    stdout: Stdout,
    stderr: Stderr,
    term_controller: TermController,
}

#[derive(Clone)]
struct ProgressState {
    index: usize,
    state: usize,
    max_val: usize,
}

impl<'a> ProgressBar<'a> {
    fn new(init_message: &'a str, final_message: &'a str) -> Self {
        Self {
            init_message,
            final_message,
            states: Vec::new(),
            stdout: io::stdout(),
            stderr: io::stderr(),
            term_controller: TermController::new(),
        }
    }

    fn add_bar(&mut self, max_val: usize) -> ProgressState {
        let mut handle = self.stdout.lock();

        // Write new line
        handle.write_all(b"\n").unwrap();
        self.stdout.flush().unwrap();

        let state = ProgressState {
            index: self.states.len(),
            state: 0,
            max_val,
        };

        self.states.push(state.clone());

        state
    }

    fn did_all_finished(&self) -> bool {
        for _state in self.states.clone() {
            // println!("{:?}", state.lock().unwrap());
        }
        true
    }

    fn get_bar_cursor_position(&self, bar: &ProgressState) -> usize {
        self.states.len() - bar.index
    }

    fn increment_and_draw(&mut self, progress_state: &mut ProgressState, by: usize) {
        if progress_state.state == progress_state.max_val {
            return;
        }

        let mut handle = self.stdout.lock();

        // Only if drawing multiple bars
        // (moves cursor to it's bar's position)
        handle
            .write_all(
                format!(
                    "\x1B[s\x1B[{}A\r",
                    self.get_bar_cursor_position(progress_state)
                )
                .as_bytes(),
            )
            .unwrap();

        if progress_state.state + by >= progress_state.max_val {
            progress_state.finish();
        } else {
            progress_state.state += by;
        }

        let eta_pos =
            self.term_controller.columns as usize - (progress_state.state.to_string().len() + 12);
        handle
            .write_all(
                format!(
                    "{} {:eta_pos$} --:-- ETA",
                    progress_state.state,
                    "",
                    eta_pos = eta_pos
                )
                .as_bytes(),
            )
            .unwrap();

        // Only if drawing multiple bars
        // (returns cursor to it's position back)
        handle.write_all("\x1B[u\r".as_bytes()).unwrap();

        self.stdout.flush().unwrap();
    }
}

impl ProgressState {
    fn finish(&mut self) {
        self.state = self.max_val;
    }

    fn is_completed(&self) -> bool {
        self.state == self.max_val
    }
}

fn main() -> io::Result<()> {
    let mpbar = Arc::new(Mutex::new(ProgressBar::new("Starting", "Finished")));

    let pbar = mpbar.clone();
    let mut pstate = mpbar.lock().unwrap().add_bar(1000);
    thread::spawn(move || loop {
        if pstate.is_completed() {
            break;
        }

        pbar.lock().unwrap().increment_and_draw(&mut pstate, 12);
        thread::sleep(Duration::from_millis(61));
    });

    let pbar = mpbar.clone();
    let mut pstate = pbar.lock().unwrap().add_bar(5000);
    thread::spawn(move || loop {
        if pstate.is_completed() {
            break;
        }

        pbar.lock().unwrap().increment_and_draw(&mut pstate, 250);
        thread::sleep(Duration::from_millis(61));
    });

    let pbar = mpbar.clone();
    let mut pstate = pbar.lock().unwrap().add_bar(1250);
    thread::spawn(move || loop {
        if pstate.is_completed() {
            break;
        }

        pbar.lock().unwrap().increment_and_draw(&mut pstate, 70);
        thread::sleep(Duration::from_millis(61));
    });

    let pbar = mpbar.clone();
    let mut pstate = pbar.lock().unwrap().add_bar(10000);
    thread::spawn(move || loop {
        if pstate.is_completed() {
            break;
        }

        pbar.lock().unwrap().increment_and_draw(&mut pstate, 350);
        thread::sleep(Duration::from_millis(61));
    });

    let mut buf: String = String::new();
    io::stdin().read_line(&mut buf).unwrap();

    // Get terminal size
    let (columns, rows) = TermController::get_columns_and_rows();
    println!("Width: {} Height: {}", columns, rows);

    let (x, y) = TermController::get_xy_pixels();
    println!("X: {} Y: {}", x, y);

    Ok(())
}

// TODO
// Keep printing the duration
// and let the progress state trigger manually
//
//
// optimize the runtime performance (try and measure mpsc performance)
