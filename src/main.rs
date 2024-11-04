use std::env;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{poll, read, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

fn main() -> io::Result<()> {
    // 構造体を使用して出力ハンドルを共有
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;

    // ターミナルをrawモードに設定
    enable_raw_mode()?;

    // アプリケーションの終了フラグ
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // ステータスバーを更新するスレッド
    let status_handle = thread::spawn(move || {
        while r.load(Ordering::SeqCst) {
            // ステータスバーの描画
            let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let status = format!(" Time: {} ", now);

            // ターミナルのサイズを取得
            let (cols, rows) = match crossterm::terminal::size() {
                Ok((c, r)) => (c, r),
                Err(_) => (80, 24),
            };

            // ステータスバーを描画
            let mut stdout = io::stdout();
            execute!(
                stdout,
                MoveTo(0, rows - 1),
                SetForegroundColor(Color::Black),
                SetBackgroundColor(Color::White),
                Clear(ClearType::CurrentLine),
                Print(status),
                ResetColor
            )
            .unwrap();

            // 1秒待つ
            thread::sleep(Duration::from_secs(1));
        }
    });

    // シェルを実行
    let result = run_shell();

    // アプリケーションの終了を通知
    running.store(false, Ordering::SeqCst);
    status_handle.join().unwrap();

    // 入力イベントを確認し、必要なら終了
    // ここではシェルが終了したら終了するため、特にイベント処理は行いません

    // ターミナルの復元
    disable_raw_mode()?;
    execute!(stdout, Show, LeaveAlternateScreen)?;

    result
}

fn run_shell() -> io::Result<()> {
    let shell = if cfg!(target_os = "windows") {
        "powershell".to_string()
    } else {
        env::var("SHELL").unwrap_or_else(|_| String::from("/bin/sh"))
    };

    let mut child = Command::new(shell)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    child.wait()?;
    Ok(())
}
