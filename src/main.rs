use std::env;
use std::io;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use ctrlc;
use dotenvy::dotenv;

fn main() -> io::Result<()> {
    dotenv().ok();
    let shell = env::var("SHELL").unwrap_or_else(|_| {
        if cfg!(target_os = "windows") {
            String::from("C:\\Windows\\system32\\cmd.exe")
        } else {
            String::from("/bin/sh")
        }
    });

    let child = Arc::new(Mutex::new(
        Command::new(shell)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?,
    ));

    let child_clone = Arc::clone(&child);
    ctrlc::set_handler(move || {
        // Ctrl+C を受け取ったときに子プロセスにシグナルを送る
        let _ = child_clone.lock().unwrap().kill();
    }).expect("Error setting Ctrl-C handler");

    child.lock().unwrap().wait()?;
    Ok(())
}
