use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
    ExecutableCommand,
};
use std::{
    env,
    io::{self, Write},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};
use ctrlc;
use dotenvy::dotenv;

fn main() -> io::Result<()> {
    dotenv().ok();

    // Determine the shell to use
    let shell = env::var("SHELL").unwrap_or_else(|_| {
        if cfg!(target_os = "windows") {
            String::from("C:\\Windows\\system32\\cmd.exe")
        } else {
            String::from("/bin/sh")
        }
    });

    // Spawn the shell process
    let child = Arc::new(Mutex::new(
        Command::new(&shell)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?,
    ));

    // Handle Ctrl-C to kill the child process
    let child_clone = Arc::clone(&child);
    ctrlc::set_handler(move || {
        let _ = child_clone.lock().unwrap().kill();
    })
    .expect("Error setting Ctrl-C handler");

    // Set up terminal in raw mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(crossterm::terminal::EnterAlternateScreen)?;
    stdout.execute(crossterm::cursor::Hide)?;

    // Shared state to manage prefix mode
    let prefix_mode = Arc::new(Mutex::new(false));
    let prefix_mode_clone = Arc::clone(&prefix_mode);

    // Spawn a thread to handle input
    let handle = thread::spawn(move || -> io::Result<()> {
        let mut value = child.lock();
        loop {
            // Wait for an event with a timeout
            if event::poll(std::time::Duration::from_millis(500))? {
                if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                    let mut pm = prefix_mode_clone.lock().unwrap();
                    if *pm {
                        // We are in prefix mode; handle commands
                        match (code, modifiers) {
                            (KeyCode::Insert, KeyModifiers::CONTROL) => {
                                // Example: Ctrl+C after prefix
                                println!("Prefix + Ctrl+C detected.");
                                // Add your command handling here
                            }
                            _ => {
                                println!("Unknown prefix command.");
                            }
                        }
                        *pm = false; // Exit prefix mode after handling
                    } else {
                        // Not in prefix mode; check for prefix key
                        if code == KeyCode::Char('b') && modifiers.contains(KeyModifiers::CONTROL) {
                            *pm = true;
                            println!("Prefix detected. Awaiting command...");
                        } else {
                            // Forward the key to the shell if not a prefix
                            // This part may require more sophisticated handling
                            // depending on how you want to integrate with the shell
                        }
                    }
                }
            }

            // Optionally, you can check if the child process has exited
            // and break the loop if it has
            let mut child_guard = value.unwrap();
            if child_guard.try_wait()?.is_some() {
                break;
            }
        }
        Ok(())
    });

    // Wait for the input handling thread to finish
    handle.join().unwrap()?;

    // Restore terminal
    stdout.execute(crossterm::cursor::Show)?;
    stdout.execute(crossterm::terminal::LeaveAlternateScreen)?;
    disable_raw_mode()?;

    // Wait for the child process to exit
    child.lock().unwrap().wait()?;
    Ok(())
}
