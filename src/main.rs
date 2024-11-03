use std::env;
use std::io;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use ctrlc;
use dotenvy::dotenv;
use crossterm::ExecutableCommand;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use std::io::stdout;

fn main() -> io::Result<()> {
    dotenv().ok();
    let shell = env::var("SHELL").unwrap_or_else(|_| {
        if cfg!(target_os = "windows") {
            String::from("C:\\Windows\\system32\\cmd.exe")
        } else {
            String::from("/bin/sh")
        }
    });

    stdout().execute(EnterAlternateScreen)?;

    let child = Arc::new(Mutex::new(
        Command::new(shell)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?,
    ));

    // ^C
    let child_clone = Arc::clone(&child);
    ctrlc::set_handler(move || {
        let _ = child_clone.lock().unwrap().kill();
    }).expect("Error setting Ctrl-C handler");

    let status = child.lock().unwrap().wait();

    stdout().execute(LeaveAlternateScreen)?;

    // return process exit status
    status.map(|_| ())
}
