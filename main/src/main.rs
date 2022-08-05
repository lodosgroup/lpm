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

#[derive(Debug)]
struct ProgressState {
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
        let state = Arc::new(Mutex::new(ProgressState {
            state: 0,
            max_val,
            stdout: io::stdout(),
            stderr: io::stderr(),
        }));

        self.states.push(state.clone());

        state
    }

    fn did_all_finished(&self) -> bool {
        for state in self.states.clone() {
            println!("{:?}", state.lock().unwrap());
        }
        true
    }
}

impl ProgressState {
    fn increment(&mut self, by: usize) {
        if self.state == self.max_val {
            return;
        }

        let mut handle = self.stdout.lock();
        //        if self.state == 0 {
        //            // \n
        //            handle.write_all(&[10]).unwrap();
        //        }
        

        if self.state + by >= self.max_val {
            self.finish();
            handle
                .write_all(format!("\r{}\n", self.state).as_bytes())
                .unwrap();
        } else {
            self.state += by;
            handle
                .write_all(format!("\r{}", self.state).as_bytes())
                .unwrap();
        }

        self.stdout.flush().unwrap();
    }

    fn finish(&mut self) {
        self.state = self.max_val;
    }

    fn is_completed(&mut self) -> bool {
        self.state == self.max_val
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
    let pstate = pbar.add_bar(1000);

    thread::spawn(move || loop {
        let mut pstate = pstate.lock().unwrap();

        if pstate.is_completed() {
            break;
        }

        pstate.increment(12);
        thread::sleep(Duration::from_millis(61));
    });
    let pstate = pbar.add_bar(50000);
    thread::spawn(move || loop {
        let mut pstate = pstate.lock().unwrap();

        if pstate.is_completed() {
            break;
        }

        pstate.increment(1);
        thread::sleep(Duration::from_millis(61));
    });
    let pstate = pbar.add_bar(150000000);
    thread::spawn(move || loop {
        let mut pstate = pstate.lock().unwrap();

        if pstate.is_completed() {
            break;
        }

        pstate.increment(5);
        thread::sleep(Duration::from_millis(61));
    });
    let pstate = pbar.add_bar(12222);
    thread::spawn(move || loop {
        let mut pstate = pstate.lock().unwrap();

        if pstate.is_completed() {
            break;
        }

        pstate.increment(30);
        thread::sleep(Duration::from_millis(61));
    });

    pbar.did_all_finished();

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
