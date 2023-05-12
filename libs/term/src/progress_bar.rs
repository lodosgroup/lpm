use crate::controller::TermController;

use std::io::{self, Stdout, Write};

pub struct ProgressBar<'a> {
    init_message: &'a str,
    final_message: &'a str,
    states: Vec<(usize, usize)>,
    stdout: Stdout,
    term_controller: TermController,
    ready: bool,
}

impl<'a> ProgressBar<'a> {
    pub fn new(init_message: &'a str, final_message: &'a str) -> Self {
        Self {
            init_message,
            final_message,
            states: Vec::new(),
            stdout: io::stdout(),
            term_controller: TermController::new(),
            ready: false,
        }
    }

    pub fn initialize(&mut self) {
        let mut handle = self.stdout.lock();

        handle
            .write_all(format!("{}\n", self.init_message).as_bytes())
            .unwrap();

        self.ready = true
    }

    pub fn finalize(&mut self) {
        // TODO
        // do error handling here
        if !self.ready {}

        let mut handle = self.stdout.lock();

        handle
            .write_all(format!("{}\n", self.final_message).as_bytes())
            .unwrap();

        self.set_initial_values();
    }

    fn set_initial_values(&mut self) {
        self.ready = false;
        self.states = Vec::new();
    }

    pub fn add_bar(&mut self, max_val: usize) -> usize {
        // TODO
        // do error handling here
        if !self.ready {}

        let mut handle = self.stdout.lock();

        // Write new line
        handle.write_all(b"\n").unwrap();
        handle.flush().unwrap();

        let state = (0, max_val);

        self.states.push(state);

        self.states.len() - 1
    }

    pub fn progress_completed(&self) -> bool {
        // TODO
        // do error handling here
        if !self.ready {}

        for (state, max_val) in self.states.iter() {
            if state != max_val {
                return false;
            }
        }

        true
    }

    fn get_bar_cursor_position(&self, state_id: usize) -> usize {
        self.states.len() - state_id
    }

    pub fn is_state_completed(&self, state_id: usize) -> bool {
        // TODO
        // do error handling here
        if !self.ready {}

        self.states[state_id].0 == self.states[state_id].1
    }

    pub fn increment_and_draw(&mut self, state_id: usize, by: usize) {
        // TODO
        // do error handling here
        if !self.ready {}

        if self.states[state_id].0 == self.states[state_id].1 {
            return;
        }

        let mut handle = self.stdout.lock();

        handle
            .write_all(
                format!("\x1B[s\x1B[{}A\r", self.get_bar_cursor_position(state_id)).as_bytes(),
            )
            .unwrap();

        if self.states[state_id].0 + by >= self.states[state_id].1 {
            self.states[state_id].0 = self.states[state_id].1;
        } else {
            self.states[state_id].0 += by;
        }

        let eta_pos =
            self.term_controller.columns() - (self.states[state_id].0.to_string().len() + 12);
        handle
            .write_all(
                format!(
                    "{} {:eta_pos$} --:-- ETA",
                    self.states[state_id].0,
                    "",
                    eta_pos = eta_pos
                )
                .as_bytes(),
            )
            .unwrap();

        handle.write_all("\x1B[u\r".as_bytes()).unwrap();
        handle.flush().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        thread,
        time::Duration,
    };

    use super::*;

    #[test]
    fn test_progress_bar() {
        let mpbar = Arc::new(Mutex::new(ProgressBar::new(
            "=====   Progress bar test started   =====",
            "=====   Progress bar test finished  =====",
        )));
        mpbar.lock().unwrap().initialize();

        let pstate_id = mpbar.lock().unwrap().add_bar(1000);
        let pbar = mpbar.clone();
        let t1 = thread::spawn(move || loop {
            if pbar.lock().unwrap().is_state_completed(pstate_id) {
                break;
            }

            pbar.lock().unwrap().increment_and_draw(pstate_id, 12);
            thread::sleep(Duration::from_millis(34));
        });

        let pstate_id = mpbar.lock().unwrap().add_bar(5000);
        let pbar = mpbar.clone();
        let t2 = thread::spawn(move || loop {
            if pbar.lock().unwrap().is_state_completed(pstate_id) {
                break;
            }

            pbar.lock().unwrap().increment_and_draw(pstate_id, 250);
            thread::sleep(Duration::from_millis(61));
        });

        let pstate_id = mpbar.lock().unwrap().add_bar(1250);
        let pbar = mpbar.clone();
        let t3 = thread::spawn(move || loop {
            if pbar.lock().unwrap().is_state_completed(pstate_id) {
                break;
            }

            pbar.lock().unwrap().increment_and_draw(pstate_id, 70);
            thread::sleep(Duration::from_millis(61));
        });

        let pstate_id = mpbar.lock().unwrap().add_bar(10000);
        let pbar = mpbar.clone();
        let t4 = thread::spawn(move || loop {
            if pbar.lock().unwrap().is_state_completed(pstate_id) {
                break;
            }

            pbar.lock().unwrap().increment_and_draw(pstate_id, 350);
            thread::sleep(Duration::from_millis(61));
        });

        for t in [t1, t2, t3, t4] {
            t.join().unwrap();
        }

        let mut mpbar = mpbar.lock().unwrap();

        assert!(mpbar.ready);
        assert!(!mpbar.states.is_empty());

        mpbar.finalize();
        assert!(!mpbar.ready);
        assert!(mpbar.states.is_empty());

        assert!(mpbar.progress_completed());
    }
}
