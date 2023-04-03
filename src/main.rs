use actions::run_all;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use serde::{Deserialize, Serialize};
use std::{fs, thread};
use std::time::Duration;
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Row, Table, Tabs},
    Frame, Terminal,
};
use log::{info};


mod actions;

const DB_PATH: &str = "./data.json";

#[derive(Serialize, Deserialize, Clone)]
pub struct Test {
    id: usize,
    name: String,
    status: String,
    log: String,
}

// impl Copy for Test {
// 	fn copy(self) -> Self {
		
// 	}
// }

#[derive(Serialize, Deserialize, Clone)]
struct Data {
    commands: Vec<String>,
    tests: Vec<Test>,
    test_path: String,
    exec_name: String,
}

pub struct App<'a> {
    pub titles: Vec<&'a str>,
    pub actions: Vec<String>,
    pub index: usize,
    pub test_list: Vec<Test>,
    pub test_list_state: ListState,
    pub cmd_list_state: ListState,

	pub valgrind_enabled: bool,
    pub test_path: String,
    pub exec_name: String,
}

impl<'a> App<'static> {
    fn new() -> App<'static> {
        let mut app: App<'static> = App {
            titles: vec!["Test", "Menu", "Tab2", "Tab3"],
            actions: Vec::new(),
            index: 0,
            test_list: Vec::new(),
            test_list_state: ListState::default(),
            cmd_list_state: ListState::default(),
			valgrind_enabled: false,
            test_path: String::new(),
            exec_name: String::new(),
        };
        app.read_data();
        app.test_list_state.select(Some(0));
        app.cmd_list_state.select(Some(0));
        app
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }

    pub fn read_data(&mut self) {
        let db_content = fs::read_to_string(DB_PATH).unwrap();
        let json: Data = serde_json::from_str::<Data>(&db_content).unwrap();
        self.test_list = json.tests;
        self.actions = json.commands;
        self.test_path = json.test_path;
        self.exec_name = json.exec_name;

		for index in 0..self.test_list.len() {
			info!("Addres in main is {:p}", &self.test_list[index].status);
		}
    }
}

fn main() -> Result<(), Box<dyn Error>> {
	log4rs::init_file("logging_config.yaml", Default::default()).unwrap();

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App<'static>,
    _tick_rate: Duration,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Right => app.next(),
                KeyCode::Left => app.previous(),
                KeyCode::Down => {
                    if let Some(selected) = app.test_list_state.selected() {
                        if selected >= app.test_list.len() - 1 {
                            app.test_list_state.select(Some(0));
                        } else {
                            app.test_list_state.select(Some(selected + 1));
                        }
                    }
                }
                KeyCode::Up => {
                    if let Some(selected) = app.test_list_state.selected() {
                        if selected > 0 {
                            app.test_list_state.select(Some(selected - 1));
                        } else {
                            app.test_list_state.select(Some(app.test_list.len() - 1));
                        }
                    }
                }
                KeyCode::Enter => {
                    run_all(&mut app);
                    info!("Started test");
                }
                KeyCode::Char('r') => {
                    thread::scope(|s| {
                        s.spawn(|| 	run_all(&mut app));
                    });
                    
                    info!("Started all tests");
                }
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);

    let titles = app
        .titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Spans::from(vec![
                Span::styled(
                    first,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::UNDERLINED),
                ),
                Span::styled(rest, Style::default().fg(Color::Green)),
            ])
        })
        .collect();

    let tabs = Tabs::new(titles)
        .select(app.index)
        .block(Block::default().title("Menu").borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .divider(Span::raw("|"));

    f.render_widget(tabs, chunks[0]);
    match app.index {
        0 => {
            let test_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                .split(chunks[1]);
            let (left, right) = render_test_list(app);
            f.render_stateful_widget(left, test_layout[0], &mut app.test_list_state);
            f.render_widget(right, test_layout[1]);
        }
        1 => {
            let test_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
                .split(chunks[1]);
            let (left, right) = render_commands_list(app);
            f.render_stateful_widget(left, test_layout[0], &mut app.test_list_state);
            f.render_widget(right, test_layout[1]);
        },
        // 2 => Block::default().title("Inner 2").borders(Borders::ALL),
        // 3 => Block::default().title("Inner 3").borders(Borders::ALL),
        _ => unreachable!(),
    }
    // f.render_widget(inner, chunks[1]);
}

fn render_test_list<'a>(app: &App) -> (List<'static>, Table<'static>) {
    let tests: Vec<ListItem> = app
        .test_list
        .iter()
        .map(|test| {
            // Colorcode the level depending on its type
            let style = match test.status.as_str() {
                "0" => Style::default().fg(Color::Gray),
                "10" => Style::default().fg(Color::Green),
                "RUNNING" => Style::default().fg(Color::Red),
                "CRASHED" => Style::default().fg(Color::Blue),
                _ => Style::default(),
            };
            // Add a example datetime and apply proper spacing between them
            let header = Spans::from(vec![
                Span::raw(test.name.to_string()),
                Span::raw(" ".repeat(4)),
                Span::styled(test.status.to_string(), style),
            ]);

            ListItem::new(header)
        })
        .collect();

    let test_list = List::new(tests)
        .highlight_style(
            Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL).title("Tests"))
        .start_corner(Corner::TopLeft);

    let selected_test = app
        .test_list
        .get(
            app.test_list_state
                .selected()
                .expect("there is always a selected test"),
        )
        .expect("exists")
        .clone();

    let test_detail = Table::new(vec![Row::new(vec![
        Cell::from(Span::raw(selected_test.name)),
        Cell::from(Span::raw(selected_test.status)),
        Cell::from(Span::raw(selected_test.log)),
    ])])
    .header(Row::new(vec![
        Cell::from(Span::styled(
            "Name",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Score",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Log",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ]))
    .widths(&[
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(80),
    ])
    .block(Block::default().borders(Borders::ALL).title("Details"));

    (test_list, test_detail)
    // f.render_widget(events_list, chunks[1]);
}

fn render_commands_list<'a>(app: &App) -> (List<'static>, Table<'static>) {
    let commands: Vec<ListItem> = app
        .actions
        .iter()
        .map(|title| {
            // Colorcode the level depending on its type
            // Add a example datetime and apply proper spacing between them
            let header = Spans::from(vec![
                Span::raw(title.to_string()),
            ]);

            ListItem::new(header)
        })
        .collect();

    let cmd_list = List::new(commands)
        .highlight_style(
            Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL).title("Tests"))
        .start_corner(Corner::TopLeft);

    // let selected_cmd = app
    //     .titles
    //     .get(
    //         app.cmd_list_state
    //             .selected()
    //             .expect("there is always a selected title"),
    //     )
    //     .expect("exists")
    //     .clone();

    let cmd_detail = Table::new(vec![Row::new(vec![
        ""
    ])])
    .header(Row::new(vec![
        Cell::from(Span::styled(
            "Name",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Score",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Log",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ]))
    .widths(&[
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(80),
    ])
    .block(Block::default().borders(Borders::ALL).title("Details"));

    (cmd_list, cmd_detail)
    // f.render_widget(events_list, chunks[1]);
}
