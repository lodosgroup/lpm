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

    let pstate_id = mpbar.lock().unwrap().add_bar(1000);
    let pbar = mpbar.clone();
    thread::spawn(move || loop {
        if pbar.lock().unwrap().is_state_completed(pstate_id) {
            break;
        }

        pbar.lock().unwrap().increment_and_draw(pstate_id, 12);
        thread::sleep(Duration::from_millis(34));
    });

    let pstate_id = mpbar.lock().unwrap().add_bar(5000);
    let pbar = mpbar.clone();
    thread::spawn(move || loop {
        if pbar.lock().unwrap().is_state_completed(pstate_id) {
            break;
        }

        pbar.lock().unwrap().increment_and_draw(pstate_id, 250);
        thread::sleep(Duration::from_millis(61));
    });

    let pstate_id = mpbar.lock().unwrap().add_bar(1250);
    let pbar = mpbar.clone();
    thread::spawn(move || loop {
        if pbar.lock().unwrap().is_state_completed(pstate_id) {
            break;
        }

        pbar.lock().unwrap().increment_and_draw(pstate_id, 70);
        thread::sleep(Duration::from_millis(61));
    });

    let pstate_id = mpbar.lock().unwrap().add_bar(10000);
    let pbar = mpbar.clone();
    thread::spawn(move || loop {
        if pbar.lock().unwrap().is_state_completed(pstate_id) {
            break;
        }

        pbar.lock().unwrap().increment_and_draw(pstate_id, 350);
        thread::sleep(Duration::from_millis(61));
    });

    loop {
        if mpbar.lock().unwrap().did_all_finished() {
            return Ok(());
        }
    }
}
