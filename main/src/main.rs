#![allow(dead_code)] // for debugging

use std::{
    io::{self, Stderr, Stdout, Write},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

struct ProgressBar<'a> {
    init_message: &'a str,
    final_message: &'a str,
    states: Vec<Arc<Mutex<ProgressState>>>,
}

struct ProgressState {
    index: usize,
    state: usize,
    max_val: usize,
    stdout: Stdout,
    stderr: Stderr,
}

impl<'a> ProgressBar<'a> {
    fn new(init_message: &'a str, final_message: &'a str) -> Self {
        Self {
            init_message,
            final_message,
            states: Vec::new(),
        }
    }

    fn add_bar(&mut self, max_val: usize) -> Arc<Mutex<ProgressState>> {
        let mut stdout = io::stdout();
        let mut handle = stdout.lock();

        // Write new line
        handle.write_all(b"\n").unwrap();
        stdout.flush().unwrap();

        let state = Arc::new(Mutex::new(ProgressState {
            index: self.states.len(),
            state: 0,
            max_val,
            stdout,
            stderr: io::stderr(),
        }));

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

        let mut handle = progress_state.stdout.lock();

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
            handle
                .write_all(format!("{}", progress_state.state).as_bytes())
                .unwrap();
        } else {
            progress_state.state += by;
            handle
                .write_all(format!("{}", progress_state.state).as_bytes())
                .unwrap();
        }

        // Only if drawing multiple bars
        // (returns cursor to it's position back)
        handle.write_all("\x1B[u\r".as_bytes()).unwrap();

        progress_state.stdout.flush().unwrap();
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
    let pstate = mpbar.lock().unwrap().add_bar(1000);
    thread::spawn(move || loop {
        let mut pstate = pstate.lock().unwrap();

        if pstate.is_completed() {
            break;
        }

        pbar.lock().unwrap().increment_and_draw(&mut pstate, 12);
        thread::sleep(Duration::from_millis(61));
    });

    let pbar = mpbar.clone();
    let pstate = pbar.lock().unwrap().add_bar(5000);
    thread::spawn(move || loop {
        let mut pstate = pstate.lock().unwrap();

        if pstate.is_completed() {
            break;
        }

        pbar.lock().unwrap().increment_and_draw(&mut pstate, 250);
        thread::sleep(Duration::from_millis(61));
    });

    let pbar = mpbar.clone();
    let pstate = pbar.lock().unwrap().add_bar(1250);
    thread::spawn(move || loop {
        let mut pstate = pstate.lock().unwrap();

        if pstate.is_completed() {
            break;
        }

        pbar.lock().unwrap().increment_and_draw(&mut pstate, 70);
        thread::sleep(Duration::from_millis(61));
    });

    let pbar = mpbar.clone();
    let pstate = pbar.lock().unwrap().add_bar(3000);
    thread::spawn(move || loop {
        let mut pstate = pstate.lock().unwrap();

        if pstate.is_completed() {
            break;
        }

        pbar.lock().unwrap().increment_and_draw(&mut pstate, 350);
        thread::sleep(Duration::from_millis(61));
    });

    let mut buf: String = String::new();
    io::stdin().read_line(&mut buf).unwrap();

    Ok(())
}

// TODO
// Keep printing the duration
// and let the progress state trigger manually
//
//
// single or seperated sterr/stdouts? -> Single
//
// Position calculation via `\x1B[s\x1B[{position}A\r`
