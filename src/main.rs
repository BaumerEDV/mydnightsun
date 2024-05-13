use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Terminal},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Paragraph, Wrap},
};
use regex::Regex;
use std::{
    cmp::{max, min},
    fs,
    io::stdout,
    u16, usize,
};

fn main() -> Result<(), String> {
    let filters = vec![
        Filter {
            regex: Regex::new(".*").unwrap(),
            foreground_color: Some(Color::Black),
            background_color: Some(Color::White),
        },
        Filter {
            regex: Regex::new("def").unwrap(),
            foreground_color: Some(Color::Green),
            background_color: Some(Color::Red),
        },
    ];

    let invocation_configuration = parse_args(std::env::args())?;
    let log = open_and_parse_log(&invocation_configuration.target_logfile)?;

    let log = log
        .lines()
        .map(FilteredLine::from)
        .map(|line| {
            let mut result = line;
            for filter in &filters {
                result = filter.apply(result);
            }
            result
        })
        .filter(|line| !line.filtered_out)
        .collect();

    //let mut model = Model::new(log.lines().collect());
    let mut model = Model::new(log);

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
                                .map(|line| {
                                    Line::styled(line.text, {
                                        let mut style = Style::default();
                                        if line.foreground_color.is_some() {
                                            style = style.fg(line.foreground_color.unwrap());
                                        }
                                        if line.background_color.is_some() {
                                            style = style.bg(line.background_color.unwrap());
                                        }
                                        style
                                    })
                                })
                                .collect::<Vec<_>>(),
                        )
                        .scroll((
                            0,
                            u16::try_from(model.text_offset_horizontal).unwrap_or(u16::MAX),
                        )) // can't scroll vertically
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

    //for line in log.lines() {
    //    println!("{line}");
    //}

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
    log: Vec<FilteredLine<'a>>,
    text_offset_vertical: usize,
    text_offset_horizontal: usize,
    line_wrapping: bool,
}

impl<'a> Model<'a> {
    fn new(log: Vec<FilteredLine<'a>>) -> Self {
        Self {
            log,
            text_offset_vertical: 0,
            text_offset_horizontal: 0,
            line_wrapping: false,
        }
    }

    fn get_screen_slice(&self, length: usize) -> &[FilteredLine] {
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
        self.text_offset_horizontal += amount;
    }

    fn toggle_line_wrapping(&mut self) {
        self.line_wrapping = !self.line_wrapping;
    }
}

struct FilteredLine<'a> {
    text: &'a str,
    filtered_out: bool,
    foreground_color: Option<Color>,
    background_color: Option<Color>,
}

impl<'a> From<&'a str> for FilteredLine<'a> {
    fn from(value: &'a str) -> Self {
        FilteredLine {
            text: value,
            filtered_out: true,
            foreground_color: None,
            background_color: None,
        }
    }
}

struct Filter {
    regex: Regex,
    foreground_color: Option<Color>,
    background_color: Option<Color>,
}

impl Filter {
    fn apply<'b>(&self, mut line: FilteredLine<'b>) -> FilteredLine<'b> {
        if self.regex.is_match(line.text) {
            line.filtered_out = false;
            line.foreground_color = self.foreground_color.or(line.foreground_color);
            line.background_color = self.background_color.or(line.background_color);
        }
        line
    }
}
