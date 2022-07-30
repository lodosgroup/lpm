use std::{
    io::{self, Write},
    sync::Arc,
    thread::sleep,
    time::Duration,
};

struct ProgressBar<'a> {
    init_message: &'a str,
    final_message: &'a str,
    states: Vec<Arc<ProgressState>>,
    status: Arc<bool>,
}

struct ProgressState {
    state: u8,
    max_val: u8,
}

impl<'a> ProgressBar<'a> {
    fn new(init_message: &'a str, final_message: &'a str) -> Self {
        Self {
            init_message,
            final_message,
            states: Vec::new(),
            status: false.into(),
        }
    }

    fn add_bar(&mut self) -> Arc<ProgressState> {
        let state = Arc::new(ProgressState {
            state: 0,
            max_val: 100,
        });

        self.states.push(state.clone());

        state
    }
}

fn progress() -> io::Result<()> {
    let mut stdout = io::stdout();
    let mut handle = stdout.lock();

    for val in 0..=100 {
        handle.write_all(format!("\r{}", val).as_bytes())?;
        stdout.flush()?;

        sleep(Duration::from_millis(61));
    }

    // \n
    handle.write_all(&[10])?;

    Ok(())
}

fn main() -> io::Result<()> {
    progress()
}

// TODO
// Keep printing the duration
// and let the progress state trigger manually
//
//
// while !status {
//     print duration and state
// }
