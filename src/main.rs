use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Stylize, Terminal},
    widgets::{self, Paragraph},
};
use std::{fs, io::stdout};

fn main() -> Result<(), String> {
    let invocation_configuration = parse_args(std::env::args())?;
    let log = open_and_parse_log(&invocation_configuration.target_logfile)?;
    let log: String = log
        .lines()
        .filter(|s| s.contains("bub"))
        .map(|s| s.to_owned() + "\n")
        .collect();

    stdout()
        .execute(EnterAlternateScreen)
        .map_err(|e| e.to_string())?;
    enable_raw_mode().map_err(|e| e.to_string())?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).map_err(|e| e.to_string())?;
    terminal.clear().map_err(|e| e.to_string())?;

    loop {
        terminal
            .draw(|frame| {
                let area = frame.size();
                frame.render_widget(Paragraph::new(&*log), area);
            })
            .map_err(|e| e.to_string())?;

        if event::poll(std::time::Duration::from_millis(16)).map_err(|e| e.to_string())? {
            if let event::Event::Key(key) = event::read().map_err(|e| e.to_string())? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    for line in log.lines() {
        println!("{line}");
    }

    stdout()
        .execute(LeaveAlternateScreen)
        .map_err(|e| e.to_string())?;
    disable_raw_mode().map_err(|e| e.to_string())?;
    Ok(())
}

fn parse_args(mut args: std::env::Args) -> Result<InvocationConfiguration, String> {
    args.next().expect("invocation name must be present");
    match args.next() {
        None => Err("no log file path was provided as the first argument".to_string()),
        Some(target_logfile) => Ok(InvocationConfiguration { target_logfile }),
    }
}

fn open_and_parse_log(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|_| format!("Unable to read file: {path}"))
}

struct InvocationConfiguration {
    target_logfile: String,
}
