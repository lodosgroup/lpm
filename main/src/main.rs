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
    status: Arc<bool>,
}

struct ProgressState {
    state: u8,
    max_val: u8,
    stdout: Stdout,
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

    fn add_bar(&mut self) -> Arc<Mutex<ProgressState>> {
        let state = Arc::new(Mutex::new(ProgressState {
            state: 0,
            max_val: 100,
            stdout: io::stdout(),
        }));

        self.states.push(state.clone());

        state
    }
}

impl ProgressState {
    fn start(&mut self) -> io::Result<()> {
        let mut handle = self.stdout.lock();

        while self.state <= self.max_val {
            handle.write_all(format!("\r{}", self.state).as_bytes())?;
            self.stdout.flush()?;

            thread::sleep(Duration::from_millis(61));
        }

        // \n
        handle.write_all(&[10])?;

        Ok(())
    }

    fn increment(&mut self, by: u8) {
        if self.state + by > self.max_val {
            self.finish();
        } else {
            self.state += by;
        }
    }

    fn finish(&mut self) {
        self.state = self.max_val;
    }
}

//fn progress() -> io::Result<()> {
//    let mut stdout = io::stdout();
//    let mut handle = stdout.lock();
//
//    for val in 0..=100 {
//        handle.write_all(format!("\r{}", val).as_bytes())?;
//        stdout.flush()?;
//
//        sleep(Duration::from_millis(61));
//    }
//
//    // \n
//    handle.write_all(&[10])?;
//
//    Ok(())
//}
//
//fn main() -> io::Result<()> {
//    progress()
//}

fn main() -> io::Result<()> {
    let mut pbar = ProgressBar::new("Starting", "Finished");

    let mut pstate = pbar.add_bar();

    let pstate_thread = pstate.clone();
    let pthread = thread::spawn(move || {
        let mut x = pstate_thread.lock().unwrap();
        x.start().unwrap();
    });

    loop {
        let mut x = pstate.lock().unwrap();
        x.increment(1);
    }

    pthread.join().unwrap();

    Ok(())
}

// TODO
// Keep printing the duration
// and let the progress state trigger manually
//
//
// while !status {
//     print duration and state
// }
//
// single or seperated sterr/stdouts?
//
// channels or shared reference?
