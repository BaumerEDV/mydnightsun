use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Terminal},
    text::Line,
    widgets::{Block, Paragraph, Wrap},
};
use std::{
    cmp::{max, min},
    fs,
    io::stdout,
    u16, usize,
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
                    {
                        let paragraph = Paragraph::new(
                            model
                                .get_screen_slice(area.height.into())
                                .iter()
                                .map(|line| Line::raw(*line))
                                .collect::<Vec<_>>(),
                        )
                        .scroll((0, u16::try_from(model.text_offset_horizontal).unwrap_or(u16::MAX))) // can't scroll vertically
                        // with u16 in log files, number too small, so got to handle that with
                        // get_screen_slice on the model. 65k horizontal scroll should be acceptable
                        // though
                        .block(Block::bordered().title(&*invocation_configuration.target_logfile));

                        if model.line_wrapping {
                            paragraph.wrap(Wrap { trim: false })
                        } else {
                            paragraph
                        }
                    },
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
                        KeyCode::Char('h') => model.scroll_horizontal_towars_line_start(1),
                        KeyCode::Char('l') => model.scroll_horizontal_away_from_line_start(1),
                        KeyCode::Char('w') => model.toggle_line_wrapping(),
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
    text_offset_vertical: usize,
    text_offset_horizontal: usize,
    line_wrapping: bool,
}

impl<'a> Model<'a> {
    fn new(log: Vec<&'a str>) -> Self {
        Self {
            log,
            text_offset_vertical: 0,
            text_offset_horizontal: 0,
            line_wrapping: false,
        }
    }

    fn get_screen_slice(&self, length: usize) -> &[&str] {
        let start = min(self.log.len(), self.text_offset_vertical);
        let end = min(self.log.len(), self.text_offset_vertical + length);
        &self.log[start..end]
    }

    fn scroll_lines_up(&mut self, amount: usize) {
        let amount = min(amount, self.text_offset_vertical);
        self.text_offset_vertical -= amount;
    }

    fn scroll_lines_down(&mut self, amount: usize) {
        let amount = min(amount, usize::MAX - self.text_offset_vertical);
        self.text_offset_vertical = min(
            max(self.log.len() - 1, 0),
            self.text_offset_vertical + amount,
        );
    }

    fn scroll_horizontal_towars_line_start(&mut self, amount: usize) {
        let amount = min(amount, self.text_offset_horizontal);
        self.text_offset_horizontal -= amount;
    }

    fn scroll_horizontal_away_from_line_start(&mut self, amount: usize) {
        let amount = min(amount, usize::MAX - self.text_offset_horizontal);
        self.text_offset_horizontal = min(
            max(self.log.len() - 1, 0),
            self.text_offset_horizontal + amount,
        );
    }

    fn toggle_line_wrapping(&mut self) {
        self.line_wrapping = !self.line_wrapping;
    }
}
