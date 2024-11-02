use std::env;
use std::io;
use std::process::{Command, Stdio};

fn main() -> io::Result<()> {
    // 現在のプラットフォームを確認し、対応するシェルを設定
    let shell = if cfg!(target_os = "windows") {
        "cmd"
    } else {
        &env::var("SHELL").unwrap_or_else(|_| String::from("/bin/sh"))
    };

    let mut child = Command::new(shell)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?; // 子プロセスとして仮想端末を起動

    // プロセスが終了するまで待機
    child.wait()?;
    Ok(())
}
