use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Stylize, Terminal},
    text::Line,
    widgets::{self, Paragraph},
};
use std::{
    cmp::{max, min},
    fs,
    io::stdout,
    usize,
};

fn main() -> Result<(), String> {
    let invocation_configuration = parse_args(std::env::args())?;
    let log = open_and_parse_log(&invocation_configuration.target_logfile)?;
    let mut model = Model::new(log.lines().collect());

    // setup Ratatui
    stdout()
        .execute(EnterAlternateScreen)
        .map_err(|e| e.to_string())?;
    enable_raw_mode().map_err(|e| e.to_string())?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).map_err(|e| e.to_string())?;
    terminal.clear().map_err(|e| e.to_string())?;

    loop {
        // render
        terminal
            .draw(|frame| {
                let area = frame.size();
                frame.render_widget(
                    Paragraph::new(
                        model
                            .get_screen_slice(area.height.into())
                            .iter()
                            .map(|line| Line::from(*line))
                            .collect::<Vec<_>>(),
                    ),
                    area,
                );
            })
            .map_err(|e| e.to_string())?;

        // handle input
        if event::poll(std::time::Duration::from_millis(200)).map_err(|e| e.to_string())? {
            if let event::Event::Key(key) = event::read().map_err(|e| e.to_string())? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q' | 'Q') => break,
                        KeyCode::Char('j') => model.scroll_lines_down(1),
                        KeyCode::Char('k') => model.scroll_lines_up(1),
                        KeyCode::Char('d') => model.scroll_lines_down(
                            terminal.size().map_or(0, |area| (area.height / 2).into()),
                        ),
                        KeyCode::Char('u') => model.scroll_lines_up(
                            terminal.size().map_or(0, |area| (area.height / 2).into()),
                        ),
                        _ => {}
                    };
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

#[derive(Default)]
struct Model<'a> {
    log: Vec<&'a str>,
    text_offset: usize,
}

impl<'a> Model<'a> {
    fn new(log: Vec<&'a str>) -> Self {
        Self {
            log,
            text_offset: 0,
        }
    }

    fn get_screen_slice(&self, length: usize) -> &[&str] {
        let start = min(self.log.len(), self.text_offset);
        let end = min(self.log.len(), self.text_offset + length);
        &self.log[start..end]
    }

    fn scroll_lines_up(&mut self, amount: usize) {
        let amount = min(amount, self.text_offset);
        self.text_offset -= amount;
    }

    fn scroll_lines_down(&mut self, amount: usize) {
        let amount = min(amount, usize::MAX - self.text_offset);
        self.text_offset = min(max(self.log.len() - 1, 0), self.text_offset + amount);
    }
}
