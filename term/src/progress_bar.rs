use crate::controller::TermController;
use std::io::{self, Stderr, Stdout, Write};

pub struct ProgressBar<'a> {
    init_message: &'a str,
    final_message: &'a str,
    states: Vec<ProgressState>,
    stdout: Stdout,
    stderr: Stderr,
    term_controller: TermController,
}

#[derive(Clone)]
pub struct ProgressState {
    index: usize,
    state: usize,
    max_val: usize,
}

impl<'a> ProgressBar<'a> {
    pub fn new(init_message: &'a str, final_message: &'a str) -> Self {
        Self {
            init_message,
            final_message,
            states: Vec::new(),
            stdout: io::stdout(),
            stderr: io::stderr(),
            term_controller: TermController::new(),
        }
    }

    pub fn add_bar(&mut self, max_val: usize) -> ProgressState {
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
        for state in self.states.clone() {
            if !state.is_completed() {
                return false;
            }
        }

        true
    }

    fn get_bar_cursor_position(&self, bar: &ProgressState) -> usize {
        self.states.len() - bar.index
    }

    pub fn increment_and_draw(&mut self, progress_state: &mut ProgressState, by: usize) {
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
            self.term_controller.columns() - (progress_state.state.to_string().len() + 12);
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

    pub fn is_completed(&self) -> bool {
        self.state == self.max_val
    }
}
