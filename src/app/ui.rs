use std::time::Duration;

use similar::{ChangeTag, TextDiff};
use symbols::line;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{
    Block, BorderType, Borders, Cell, List, ListItem, Paragraph, Row, Table, Tabs, Clear, Wrap,
};
use tui::{symbols, Frame};
use tui_logger::TuiLoggerWidget;

use super::actions::Actions;
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
                Constraint::Length(0),
                Constraint::Min(10),
                // Constraint::Length(3),
                Constraint::Length(12),
            ]
            .as_ref(),
        )
        .split(size);

    // Tabs
    let tabs = draw_tabs(app);
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
        .constraints([Constraint::Length(24), Constraint::Min(20)].as_ref())
        .split(body_chunks[0]);

    let (test_list, test_info, test_log, first_diff) = draw_test_list(app);

    let test_and_score_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)].as_ref())
        .split(test_layout[0]);

    rect.render_stateful_widget(
        test_list,
        test_and_score_layout[0],
        &mut app.test_list_state,
    );

    let score = draw_final_score(app);
    rect.render_widget(score, test_and_score_layout[1]);

    let info_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(20)].as_ref())
        .split(test_layout[1]);
    rect.render_widget(test_info, info_layout[0]);

    if let Some(index) = app.windows_list_state.selected() {
        if index == 1 {
            if app.log_list_state.selected().is_none() {
                app.log_list_state.select(Some(first_diff));
            }
            rect.render_stateful_widget(test_log, info_layout[1], &mut app.log_list_state);
        } else {
            app.log_list_state.select(None);
            rect.render_widget(test_log, info_layout[1]);
        }
    }

    let help = draw_help(app.actions());
    rect.render_widget(help, body_chunks[1]);

    // Duration LineGauge
    // if let Some(duration) = app.state().duration() {
    //     let duration_block = draw_duration(duration);
    //     rect.render_widget(duration_block, chunks[2]);
    // }

    // Logs
    let logs = draw_logs();
    rect.render_widget(logs, chunks[2]);

    if let Some(true) = app.state().get_checkstyle() {
        let (area, block) = draw_popup_cs(app, size, 90, 90);

        rect.render_widget(Clear, area);
        rect.render_widget(block, area);
    }
}

fn draw_popup_cs<'a>(app: &'a App, size: Rect, x: u16, y: u16) -> (Rect, Paragraph<'a>) {
    let items: Vec<_> = app.checkstyle
        .lines()
        .map(|line| {
            let style = match line {
                _ if line.contains("CHECK") => Style::default().fg(Color::Green),
                _ if line.contains("WARNING") => Style::default().fg(Color::Yellow),
                _ if line.contains("ERROR") => Style::default().fg(Color::Red),
                _ => Style::default(),
            };

            let split: Vec<&'a str> = line.split(':').collect();

            let header = Spans::from(vec![
                Span::styled(format!("{}:{}:{}:", split[0], split[1], split[2]) , style),
                Span::styled(split[3], Style::default().fg(Color::Blue)),
                Span::raw(":"),
                Span::raw(split[4]),
            ]);

            // ListItem::new(header)
            header
        }).collect();

    let list = Paragraph::new(items)
        .block(Block::default()
        .borders(Borders::ALL)
        .title("Checkstyle"))
        .wrap(Wrap { trim: true });

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - y) / 2),
                Constraint::Percentage(y),
                Constraint::Percentage((100 - y) / 2),
            ]
            .as_ref(),
        )
        .split(size);

        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage((100 - x) / 2),
                    Constraint::Percentage(x),
                    Constraint::Percentage((100 - x) / 2),
                ]
                .as_ref(),
            )
            .split(popup_layout[1])[1];

    (area, list)
}

fn draw_test_list<'a>(app: &App) -> (List<'a>, Table<'a>, List<'a>, usize) {
    let tests: Vec<ListItem> = app
        .test_list
        .iter()
        .map(|test| {
            // Colorcode the level depending on its type
            let style = match test.status.as_str() {
                "0" => Style::default().fg(Color::Gray),
                "RUNNING" => Style::default().fg(Color::Green),
                "ERROR" => Style::default().fg(Color::Red),
                "CRASHED" => Style::default().fg(Color::Blue),
                "STARTING" => Style::default().fg(Color::Blue),
                "TIMEOUT" => Style::default().fg(Color::Blue),
                "MEMLEAKS" => Style::default().fg(Color::Blue),
                _ => Style::default().fg(Color::Green),
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
    
    let style = Style::default().fg(if app.valgrind_enabled {Color::Red} else {Color::Gray});

    let test_list = List::new(tests)
        .highlight_style(
            Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL)
            .border_style(style).title("Tests"));

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
        Cell::from(Span::raw(convert_time_to_string(if app.valgrind_enabled {
            selected_test.time_valgrind
        } else {
            selected_test.time_normal
        }))),
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
            if app.valgrind_enabled {
                "Time Valgrind"
            } else {
                "Time"
            },
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ]))
    .widths(&[
        Constraint::Percentage(20),
        Constraint::Percentage(30),
        Constraint::Percentage(50),
    ])
    .block(Block::default().borders(Borders::ALL).title("Details"));

    // TODO: Don't read this file on every tick
    // let ref_file = fs::read_to_string(format!("{}ref/{}-test.ref", app.test_path, selected_test.id)).unwrap();

    let diff = TextDiff::from_lines(&app.current_ref, &selected_test.log);

    let mut index = 0;
    let mut first_diff: usize = usize::MAX;
    let log_items: Vec<ListItem> = diff
        .iter_all_changes()
        .map(|line| {
            let (sign, style) = match line.tag() {
                ChangeTag::Delete => ("-", Style::default().fg(Color::Red)),
                ChangeTag::Insert => ("+", Style::default().fg(Color::Yellow)),
                ChangeTag::Equal => (" ", Style::default().fg(Color::Gray)),
            };

            if line.tag() != ChangeTag::Equal && first_diff > index {
                first_diff = index;
            }

            index += 1;

            match line.missing_newline() {
                true => ListItem::new(Span::styled(format!("{}{}", sign, line), style)),
                false => ListItem::new(Span::styled(format!("{}{}⏎", sign, line), style)),
            }
        })
        .collect();

    let test_log = List::new(log_items)
        .highlight_style(
            Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ")
        .block(Block::default().borders(Borders::ALL).title("Test log"));

    if first_diff == usize::MAX {
        first_diff = 0;
    }

    (test_list, test_detail, test_log, first_diff)
}

fn draw_final_score<'a>(app: &App) -> Paragraph<'a> {
    let score = app.calculate_score();

    let style = match score {
        0 => Style::default().fg(Color::Red),
        100 => Style::default().fg(Color::Green),
        _ => Style::default(),
    };

    Paragraph::new(vec![Spans::from(Span::styled(
        format!("{}/100", score),
        style,
    ))])
    .alignment(Alignment::Right)
    .block(
        Block::default()
            .title("Final score")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .border_type(BorderType::Plain),
    )
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

// fn draw_duration(duration: &Duration) -> LineGauge {
//     let sec = duration.as_secs();
//     let label = format!("{}s", sec);
//     let ratio = sec as f64 / 10.0;
//     LineGauge::default()
//         .block(
//             Block::default()
//                 .borders(Borders::ALL)
//                 .title("Sleep duration"),
//         )
//         .gauge_style(
//             Style::default()
//                 .fg(Color::Cyan)
//                 .bg(Color::Black)
//                 .add_modifier(Modifier::BOLD),
//         )
//         .line_set(line::THICK)
//         .label(label)
//         .ratio(ratio)
// }

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
