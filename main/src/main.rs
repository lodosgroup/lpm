#![allow(dead_code)] // for debugging

use std::{
    io,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use term::progress_bar::ProgressBar;

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

    Ok(())
}
