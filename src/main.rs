mod app;
mod ui;
mod core;
mod providers;
mod models;
mod config;
mod utils;

use std::io;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

use app::{App, ExecutionState};
use crate::models::tool::Tool;
use crate::config::settings::Config;

// Messages from background tasks back to the event loop
enum BgMessage {
    SearchResult(Result<Vec<Tool>, String>),
    InstallDone(Result<(), String>),
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let config = Config::load();

    // TUI setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, mut rx) = mpsc::unbounded_channel::<BgMessage>();
    let mut app = App::new();

    let result = run_loop(&mut terminal, &mut app, &mut rx, &tx, &config).await;

    // TUI teardown
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    rx: &mut mpsc::UnboundedReceiver<BgMessage>,
    tx: &mpsc::UnboundedSender<BgMessage>,
    config: &Config,
) -> io::Result<()> {
    loop {
        // Drain background messages
        while let Ok(msg) = rx.try_recv() {
            match msg {
                BgMessage::SearchResult(Ok(tools)) => app.set_results(tools),
                BgMessage::SearchResult(Err(e)) => app.set_error(e),
                BgMessage::InstallDone(Ok(())) => {
                    app.execution_state = ExecutionState::Success(
                        format!("Installed successfully.")
                    );
                }
                BgMessage::InstallDone(Err(e)) => {
                    app.execution_state = ExecutionState::Error(e);
                }
            }
        }

        app.tick();

        terminal.draw(|f| {
            ui::layout::draw(f, app);
        })?;

        if !event::poll(std::time::Duration::from_millis(100))? {
            continue;
        }

        let Event::Key(key) = event::read()? else {
            continue;
        };

        if key.kind != KeyEventKind::Press {
            continue;
        }

        // Dismiss any success/error status on any keypress
        if matches!(
            app.execution_state,
            ExecutionState::Success(_) | ExecutionState::Error(_)
        ) {
            app.execution_state = ExecutionState::Idle;
        }

        // Global quit: Ctrl-C always exits
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            break;
        }

        match app.input_mode {
            app::InputMode::Search => {
                match key.code {
                    // Submit search
                    KeyCode::Enter => {
                        let q = app.query.trim().to_string();
                        if !q.is_empty() {
                            app.push_history(&q);
                            app.set_loading();
                            let tx_clone = tx.clone();
                            let query_clone = q.clone();
                            let cfg_clone = config.clone();
                            tokio::spawn(async move {
                                let res = core::search::search(&query_clone, &cfg_clone).await;
                                let msg = match res {
                                    Ok(tools) => BgMessage::SearchResult(Ok(tools)),
                                    Err(e) => BgMessage::SearchResult(Err(e.to_string())),
                                };
                                let _ = tx_clone.send(msg);
                            });
                            app.leave_search();
                        }
                    }
                    KeyCode::Esc => app.leave_search(),
                    KeyCode::Char(c) => app.add_char(c),
                    KeyCode::Backspace => app.remove_char(),
                    _ => {}
                }
            }

            app::InputMode::Normal => {
                match key.code {
                    // Quit
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,

                    // Enter search mode
                    KeyCode::Char('/') => {
                        app.enter_search();
                    }

                    // Navigation
                    KeyCode::Down | KeyCode::Char('j') => app.select_next(),
                    KeyCode::Up | KeyCode::Char('k') => app.select_prev(),

                    // Detail panel scroll
                    KeyCode::Char('d') => app.scroll_detail_down(),
                    KeyCode::Char('u') => app.scroll_detail_up(),

                    // Install
                    KeyCode::Char('i') => {
                        if let Some(tool) = app.current_tool().cloned() {
                            if !matches!(app.execution_state, ExecutionState::Installing) {
                                app.execution_state = ExecutionState::Installing;
                                let tx_clone = tx.clone();
                                tokio::task::spawn_blocking(move || {
                                    let res = core::install::install(&tool)
                                        .map_err(|e| e.to_string());
                                    let _ = tx_clone.send(BgMessage::InstallDone(res));
                                });
                            }
                        }
                    }

                    // Run
                    KeyCode::Char('r') | KeyCode::Enter => {
                        if let Some(tool) = app.current_tool().cloned() {
                            if !matches!(app.execution_state, ExecutionState::Installing) {
                                // Suspend TUI
                                disable_raw_mode()?;
                                execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                                terminal.show_cursor()?;

                                // Run tool synchronously
                                println!("Running {}...\r", tool.name);
                                let res = core::run::run(&tool);

                                // Resume TUI
                                enable_raw_mode()?;
                                execute!(terminal.backend_mut(), EnterAlternateScreen)?;
                                terminal.clear()?;

                                match res {
                                    Ok(()) => app.execution_state = ExecutionState::Success("Completed.".to_string()),
                                    Err(e) => app.execution_state = ExecutionState::Error(e),
                                }
                            }
                        }
                    }

                    _ => {}
                }
            }
        }
    }

    Ok(())
}
