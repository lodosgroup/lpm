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

struct ProgressState {
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

    pub fn add_bar(&mut self, max_val: usize) -> usize {
        let mut handle = self.stdout.lock();

        // Write new line
        handle.write_all(b"\n").unwrap();
        self.stdout.flush().unwrap();

        let state = ProgressState { state: 0, max_val };

        self.states.push(state);

        self.states.len() - 1
    }

    pub fn did_all_finished(&self) -> bool {
        for state in self.states.iter() {
            if !state.is_completed() {
                return false;
            }
        }

        true
    }

    fn get_bar_cursor_position(&self, state_id: usize) -> usize {
        self.states.len() - state_id
    }

    pub fn is_state_completed(&self, state_id: usize) -> bool {
        self.states[state_id].is_completed()
    }

    pub fn increment_and_draw(&mut self, state_id: usize, by: usize) {
        if self.states[state_id].state == self.states[state_id].max_val {
            return;
        }

        let mut handle = self.stdout.lock();

        // Only if drawing multiple bars
        // (moves cursor to it's bar's position)
        handle
            .write_all(
                format!("\x1B[s\x1B[{}A\r", self.get_bar_cursor_position(state_id)).as_bytes(),
            )
            .unwrap();

        if self.states[state_id].state + by >= self.states[state_id].max_val {
            self.states[state_id].finish();
        } else {
            self.states[state_id].state += by;
        }

        let eta_pos =
            self.term_controller.columns() - (self.states[state_id].state.to_string().len() + 12);
        handle
            .write_all(
                format!(
                    "{} {:eta_pos$} --:-- ETA",
                    self.states[state_id].state,
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
