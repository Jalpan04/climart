use crate::app::{App, ExecutionState};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

// --- Color palette ---
const BRAND_CYAN: Color = Color::Rgb(0, 210, 200);
const BRAND_MAGENTA: Color = Color::Rgb(180, 80, 220);
const ACCENT_YELLOW: Color = Color::Rgb(255, 200, 50);
const ACCENT_GREEN: Color = Color::Rgb(60, 210, 100);
const ACCENT_RED: Color = Color::Rgb(240, 70, 70);
const ACCENT_ORANGE: Color = Color::Rgb(255, 140, 50);
const BG_DARK: Color = Color::Rgb(12, 14, 20);
const BG_PANEL: Color = Color::Rgb(20, 24, 35);
const FG_DIM: Color = Color::Rgb(100, 110, 130);
const FG_NORMAL: Color = Color::Rgb(200, 210, 230);
const FG_BRIGHT: Color = Color::White;
const SOURCE_NPM: Color = ACCENT_YELLOW;
const SOURCE_PIPX: Color = Color::Rgb(50, 140, 220);
const SOURCE_BREW: Color = Color::Rgb(255, 165, 0);
const SOURCE_PKGX: Color = Color::Rgb(80, 200, 140);

pub fn draw(f: &mut Frame, app: &App) {
    // Background
    f.render_widget(
        Block::default().style(Style::default().bg(BG_DARK)),
        f.size(),
    );

    // Top-level vertical split: header | body | footer
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header / search bar
            Constraint::Min(0),    // body
            Constraint::Length(1), // status bar
            Constraint::Length(3), // footer / key hints
        ])
        .split(f.size());

    draw_header(f, app, root[0]);

    // Body: results | details
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
        .split(root[1]);

    draw_results(f, app, body[0]);
    draw_details(f, app, body[1]);

    draw_status(f, app, root[2]);
    draw_footer(f, app, root[3]);
}

// ---- Header / Search bar ----

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let (border_color, title_color) = if app.search_focused() {
        (ACCENT_YELLOW, ACCENT_YELLOW)
    } else {
        (BRAND_CYAN, BRAND_CYAN)
    };

    let prefix = if app.search_focused() {
        Span::styled("/ ", Style::default().fg(ACCENT_YELLOW).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("  ", Style::default().fg(FG_DIM))
    };

    let query_text = if app.search_focused() {
        // Show cursor at end
        Line::from(vec![
            prefix,
            Span::styled(&app.query, Style::default().fg(FG_BRIGHT)),
            Span::styled("_", Style::default().fg(ACCENT_YELLOW).add_modifier(Modifier::RAPID_BLINK)),
        ])
    } else if app.query.is_empty() {
        Line::from(vec![
            prefix,
            Span::styled("Press / to search CLI tools", Style::default().fg(FG_DIM)),
        ])
    } else {
        Line::from(vec![
            prefix,
            Span::styled(&app.query, Style::default().fg(FG_NORMAL)),
        ])
    };

    let title = Line::from(vec![
        Span::styled(" climart", Style::default().fg(BRAND_CYAN).add_modifier(Modifier::BOLD)),
        Span::styled(" | ", Style::default().fg(FG_DIM)),
        Span::styled("CLI App Store", Style::default().fg(title_color)),
        Span::styled(" ", Style::default()),
    ]);

    f.render_widget(
        Paragraph::new(query_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(border_color))
                    .title(title)
                    .title_alignment(Alignment::Left)
                    .style(Style::default().bg(BG_PANEL)),
            ),
        area,
    );
}

// ---- Results panel ----

fn draw_results(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BRAND_MAGENTA))
        .title(Line::from(vec![
            Span::styled(" Results", Style::default().fg(BRAND_MAGENTA).add_modifier(Modifier::BOLD)),
            if app.has_results() {
                Span::styled(
                    format!(" ({}) ", app.results.len()),
                    Style::default().fg(FG_DIM),
                )
            } else {
                Span::styled(" ", Style::default())
            },
        ]))
        .style(Style::default().bg(BG_PANEL));

    if app.is_loading {
        let loading = Paragraph::new(Line::from(vec![
            Span::styled("  Searching", Style::default().fg(ACCENT_YELLOW)),
            Span::styled("...", Style::default().fg(FG_DIM)),
        ]))
        .block(block);
        f.render_widget(loading, area);
        return;
    }

    if let Some(ref err) = app.error {
        let errp = Paragraph::new(Line::from(vec![
            Span::styled("  Error: ", Style::default().fg(ACCENT_RED).add_modifier(Modifier::BOLD)),
            Span::styled(err.as_str(), Style::default().fg(ACCENT_RED)),
        ]))
        .block(block)
        .wrap(Wrap { trim: true });
        f.render_widget(errp, area);
        return;
    }

    if app.results.is_empty() {
        let hint = if app.search_history.is_empty() {
            " Press / and type a search query"
        } else {
            " No results found for that query"
        };
        f.render_widget(
            Paragraph::new(Span::styled(hint, Style::default().fg(FG_DIM)))
                .block(block),
            area,
        );
        return;
    }

    let items: Vec<ListItem> = app.results.iter().enumerate().map(|(i, tool)| {
        let source_color = source_color(&tool.source);
        let source_badge = Span::styled(
            format!(" {} ", tool.source.to_uppercase()),
            Style::default().fg(Color::Black).bg(source_color).add_modifier(Modifier::BOLD),
        );
        let name_style = if i == app.selected_index {
            Style::default().fg(FG_BRIGHT).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(FG_NORMAL)
        };
        let name = Span::styled(format!(" {}", tool.name), name_style);

        ListItem::new(Line::from(vec![source_badge, name]))
    }).collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(30, 35, 55))
                .fg(FG_BRIGHT)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    if app.has_results() {
        state.select(Some(app.selected_index));
    }

    f.render_stateful_widget(list, area, &mut state);

    // Scrollbar
    if app.results.len() > area.height as usize {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█");
        let mut scrollbar_state = ScrollbarState::default()
            .content_length(app.results.len())
            .position(app.selected_index);
        f.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }
}

// ---- Details panel ----

fn draw_details(f: &mut Frame, app: &App, area: Rect) {
    let (border_color, title_sfx) = match &app.execution_state {
        ExecutionState::Error(_) => (ACCENT_RED, " ERROR "),
        ExecutionState::Success(_) => (ACCENT_GREEN, " DONE "),

        ExecutionState::Installing => (ACCENT_YELLOW, " INSTALLING "),
        ExecutionState::Idle => (BRAND_CYAN, ""),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Line::from(vec![
            Span::styled(" Details", Style::default().fg(border_color).add_modifier(Modifier::BOLD)),
            if !title_sfx.is_empty() {
                Span::styled(title_sfx, Style::default().fg(border_color).add_modifier(Modifier::RAPID_BLINK))
            } else {
                Span::raw(" ")
            },
        ]))
        .style(Style::default().bg(BG_PANEL));

    let mut lines: Vec<Line> = Vec::new();

    if let Some(tool) = app.current_tool() {
        // Name row
        lines.push(Line::from(vec![
            Span::styled("  Name     ", Style::default().fg(FG_DIM)),
            Span::styled(&tool.name, Style::default().fg(FG_BRIGHT).add_modifier(Modifier::BOLD)),
        ]));
        // Version row
        lines.push(Line::from(vec![
            Span::styled("  Version  ", Style::default().fg(FG_DIM)),
            Span::styled(&tool.version, Style::default().fg(ACCENT_YELLOW)),
        ]));
        // Source row
        let sc = source_color(&tool.source);
        lines.push(Line::from(vec![
            Span::styled("  Source   ", Style::default().fg(FG_DIM)),
            Span::styled(
                format!(" {} ", tool.source.to_uppercase()),
                Style::default().fg(Color::Black).bg(sc).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));

        // Description
        if !tool.description.is_empty() {
            lines.push(Line::from(Span::styled(
                "  Description",
                Style::default().fg(FG_DIM).add_modifier(Modifier::UNDERLINED),
            )));
            lines.push(Line::from(""));
            for desc_line in wrap_text(&tool.description, (area.width as usize).saturating_sub(4)) {
                lines.push(Line::from(Span::styled(
                    format!("  {}", desc_line),
                    Style::default().fg(FG_NORMAL),
                )));
            }
            lines.push(Line::from(""));
        }

        // Commands
        lines.push(Line::from(Span::styled(
            "  Commands",
            Style::default().fg(FG_DIM).add_modifier(Modifier::UNDERLINED),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  install  ", Style::default().fg(FG_DIM)),
            Span::styled(&tool.install_command, Style::default().fg(BRAND_CYAN)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  run      ", Style::default().fg(FG_DIM)),
            Span::styled(&tool.run_command, Style::default().fg(BRAND_CYAN)),
        ]));
    } else {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Search for CLI tools using /",
            Style::default().fg(FG_DIM),
        )));
    }

    // Execution status overlay at bottom of text
    match &app.execution_state {

        ExecutionState::Installing => {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  >>> Installing...",
                Style::default().fg(ACCENT_YELLOW).add_modifier(Modifier::BOLD),
            )));
        }
        ExecutionState::Success(msg) => {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("  >>> {}", msg),
                Style::default().fg(ACCENT_GREEN).add_modifier(Modifier::BOLD),
            )));
        }
        ExecutionState::Error(err) => {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("  >>> ERROR: {}", err),
                Style::default().fg(ACCENT_RED).add_modifier(Modifier::BOLD),
            )));
        }
        ExecutionState::Idle => {}
    }

    let total_lines = lines.len() as u16;
    let scroll = app.detail_scroll.min(total_lines.saturating_sub(1));

    f.render_widget(
        Paragraph::new(lines)
            .block(block)
            .scroll((scroll, 0))
            .wrap(Wrap { trim: false }),
        area,
    );

    // Detail scrollbar
    if total_lines > area.height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█");
        let mut scrollbar_state = ScrollbarState::default()
            .content_length(total_lines as usize)
            .position(scroll as usize);
        f.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }
}

// ---- Status bar ----

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(ref msg) = app.status_message {
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(msg.as_str(), Style::default().fg(FG_DIM)),
        ])
    } else {
        Line::from(Span::styled("  Ready", Style::default().fg(FG_DIM)))
    };

    f.render_widget(
        Paragraph::new(content).style(Style::default().bg(BG_DARK)),
        area,
    );
}

// ---- Footer / key hints ----

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let keys = if app.search_focused() {
        vec![
            key_hint("Enter", "Search"),
            key_hint("Esc", "Cancel"),
        ]
    } else {
        vec![
            key_hint("/", "Search"),
            key_hint("j/k", "Navigate"),
            key_hint("Enter/r", "Run"),
            key_hint("i", "Install"),
            key_hint("d/u", "Scroll"),
            key_hint("q", "Quit"),
        ]
    };

    let mut spans: Vec<Span> = Vec::new();
    spans.push(Span::raw("  "));
    for (idx, hint) in keys.into_iter().enumerate() {
        if idx > 0 {
            spans.push(Span::styled("  ", Style::default().fg(FG_DIM)));
        }
        spans.extend(hint);
    }

    f.render_widget(
        Paragraph::new(Line::from(spans))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Rgb(40, 45, 65)))
                    .style(Style::default().bg(BG_PANEL)),
            ),
        area,
    );
}

// ---- Helpers ----

fn key_hint(key: &str, label: &str) -> Vec<Span<'static>> {
    vec![
        Span::styled(
            format!("[{}]", key),
            Style::default().fg(ACCENT_YELLOW).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {}", label),
            Style::default().fg(FG_DIM),
        ),
    ]
}

fn source_color(source: &str) -> Color {
    match source {
        "npm" => SOURCE_NPM,
        "pipx" => SOURCE_PIPX,
        "brew" => SOURCE_BREW,
        "pkgx" => SOURCE_PKGX,
        _ => FG_DIM,
    }
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current.clone());
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}
