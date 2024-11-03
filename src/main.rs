use std::env;
use std::io;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use ctrlc;
use dotenvy::dotenv;
use crossterm::ExecutableCommand;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use std::io::stdout;

struct Pane {
    process: Arc<Mutex<std::process::Child>>,
}

impl Pane {
    fn new(shell: &str) -> io::Result<Self> {
        let process = Arc::new(Mutex::new(
            Command::new(shell)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?,
        ));
        Ok(Self { process })
    }

    fn wait(&self) -> io::Result<()> {
        self.process.lock().unwrap().wait()?;
        Ok(())
    }
}

struct Window {
    panes: Vec<Pane>,
}

impl Window {
    fn new(shell: &str) -> io::Result<Self> {
        let pane = Pane::new(shell)?;
        Ok(Self { panes: vec![pane] })
    }

    fn add_pane(&mut self, shell: &str) -> io::Result<()> {
        let pane = Pane::new(shell)?;
        self.panes.push(pane);
        Ok(())
    }

    fn run(&self) -> io::Result<()> {
        for pane in &self.panes {
            pane.wait()?;
        }
        Ok(())
    }
}

struct Session {
    windows: Vec<Window>,
}

impl Session {
    fn new(shell: &str) -> io::Result<Self> {
        let window = Window::new(shell)?;
        Ok(Self { windows: vec![window] })
    }

    fn add_window(&mut self, shell: &str) -> io::Result<()> {
        let window = Window::new(shell)?;
        self.windows.push(window);
        Ok(())
    }

    fn start(&self) -> io::Result<()> {
        for window in &self.windows {
            window.run()?;
        }
        Ok(())
    }
}

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

    let session = Session::new(&shell)?;

    // session.add_window(&shell)?;
    // session.windows[0].add_pane(&shell)?;

    let first_pane = Arc::clone(&session.windows[0].panes[0].process);
    ctrlc::set_handler(move || {
        let _ = first_pane.lock().unwrap().kill();
    }).expect("Error setting Ctrl-C handler");

    let _ = session.start();

    let status = session.windows[0].panes[0].process.lock().unwrap().wait();

    stdout().execute(LeaveAlternateScreen)?;

    // return process exit status
    status.map(|_| ())
}
