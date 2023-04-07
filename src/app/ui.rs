use std::time::Duration;

use symbols::line;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{
    Block, BorderType, Borders, Cell, LineGauge, List, ListItem, Paragraph, Row, Table, Tabs,
};
use tui::{symbols, Frame};
use tui_logger::TuiLoggerWidget;

use super::actions::Actions;
use super::state::AppState;
use crate::app::App;

pub fn draw<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let size = rect.size();
    check_size(&size);

    // Vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(10),
                // Constraint::Length(3),
                Constraint::Length(12),
            ]
            .as_ref(),
        )
        .split(size);

    // Tabs
    let tabs = draw_tabs(&app);
    rect.render_widget(tabs, chunks[0]);

    // Body & Help
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(10), Constraint::Length(32)].as_ref())
        .split(chunks[1]);

    // let body = draw_body(app.is_loading(), app.state());
    // rect.render_widget(body, body_chunks[0]);

    let test_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(body_chunks[0]);

    let (test_list, test_info, test_log) = draw_test_list(app);
    rect.render_stateful_widget(test_list, test_layout[0], &mut app.test_list_state);

    let info_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Percentage(80)].as_ref())
        .split(test_layout[1]);
    rect.render_widget(test_info, info_layout[0]);
    rect.render_widget(test_log, info_layout[1]);

    let help = draw_help(app.actions());
    rect.render_widget(help, body_chunks[1]);

    // Duration LineGauge
    if let Some(duration) = app.state().duration() {
        let duration_block = draw_duration(duration);
        rect.render_widget(duration_block, chunks[2]);
    }

    // Logs
    let logs = draw_logs();
    rect.render_widget(logs, chunks[2]);
}

fn draw_test_list<'a>(app: &App) -> (List<'a>, Table<'a>, List<'a>) {
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
                "STARTING" => Style::default().fg(Color::Blue),
                "TIMEOUT" => Style::default().fg(Color::Blue),
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
        .block(Block::default().borders(Borders::ALL).title("Tests"));

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
        Cell::from(Span::raw(convert_time_to_string(selected_test.time_normal))),
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
            "Time",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ]))
    .widths(&[
        Constraint::Percentage(20),
        Constraint::Percentage(30),
        Constraint::Percentage(50),
    ])
    .block(Block::default().borders(Borders::ALL).title("Details"));

    let log_items: Vec<ListItem> = selected_test
        .log
        .lines()
        .map(|line| ListItem::new(Span::raw(line.to_string())))
        .collect();

    let test_log = List::new(log_items)
        .highlight_style(
            Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL).title("Test log"));

    (test_list, test_detail, test_log)
}

fn draw_tabs<'a>(app: &App) -> Tabs<'a> {
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

    Tabs::new(titles)
        .select(app.selected_tab)
        .block(Block::default().title("Menu").borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .divider(Span::raw("|"))
}

fn check_size(rect: &Rect) {
    if rect.width < 52 {
        panic!("Require width >= 52, (got {})", rect.width);
    }
    if rect.height < 28 {
        panic!("Require height >= 28, (got {})", rect.height);
    }
}

fn draw_body<'a>(loading: bool, state: &AppState) -> Paragraph<'a> {
    let initialized_text = if state.is_initialized() {
        "Initialized"
    } else {
        "Not Initialized !"
    };
    let loading_text = if loading { "Loading..." } else { "" };
    let sleep_text = if let Some(sleeps) = state.count_sleep() {
        format!("Sleep count: {}", sleeps)
    } else {
        String::default()
    };
    let tick_text = if let Some(ticks) = state.count_tick() {
        format!("Tick count: {}", ticks)
    } else {
        String::default()
    };
    Paragraph::new(vec![
        Spans::from(Span::raw(initialized_text)),
        Spans::from(Span::raw(loading_text)),
        Spans::from(Span::raw(sleep_text)),
        Spans::from(Span::raw(tick_text)),
    ])
    .style(Style::default().fg(Color::LightCyan))
    .alignment(Alignment::Left)
    .block(
        Block::default()
            // .title("Body")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .border_type(BorderType::Plain),
    )
}

fn draw_duration(duration: &Duration) -> LineGauge {
    let sec = duration.as_secs();
    let label = format!("{}s", sec);
    let ratio = sec as f64 / 10.0;
    LineGauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Sleep duration"),
        )
        .gauge_style(
            Style::default()
                .fg(Color::Cyan)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .line_set(line::THICK)
        .label(label)
        .ratio(ratio)
}

fn draw_help(actions: &Actions) -> Table {
    let key_style = Style::default().fg(Color::LightCyan);
    let help_style = Style::default().fg(Color::Gray);

    let mut rows = vec![];
    for action in actions.actions().iter() {
        let mut first = true;
        for key in action.keys() {
            let help = if first {
                first = false;
                action.to_string()
            } else {
                String::from("")
            };
            let row = Row::new(vec![
                Cell::from(Span::styled(key.to_string(), key_style)),
                Cell::from(Span::styled(help, help_style)),
            ]);
            rows.push(row);
        }
    }

    Table::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .title("Help"),
        )
        .widths(&[Constraint::Length(11), Constraint::Min(20)])
        .column_spacing(1)
}

fn draw_logs<'a>() -> TuiLoggerWidget<'a> {
    TuiLoggerWidget::default()
        .style_error(Style::default().fg(Color::Red))
        .style_debug(Style::default().fg(Color::Green))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_trace(Style::default().fg(Color::Gray))
        .style_info(Style::default().fg(Color::Blue))
        .block(
            Block::default()
                .title("Logs")
                .border_type(BorderType::Plain)
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White))
}

fn convert_time_to_string(time: f64) -> String {
    let mut seconds = String::default();

    seconds.push_str(&format!("{:02}:", (time / 60f64).floor()));
    seconds.push_str(&format!("{:02}.", (time % 60f64).floor()));
    seconds.push_str(&format!("{:05}", (time.fract() * 100000.0).floor()));

    seconds
}
