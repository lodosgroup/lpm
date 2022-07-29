use std::{
    io::{self, Write},
    thread::sleep,
    time::Duration,
};

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
