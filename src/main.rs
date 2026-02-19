#![allow(dead_code)]

mod action;
mod app;
mod event;
mod filebrowser;
mod git;
mod input;
mod pty;
mod tui;
mod ui;

use anyhow::{Context, Result};
use crossterm::event::EventStream;
use futures::StreamExt;
use ratatui::widgets::Widget;
use tokio::sync::mpsc;

use action::ActiveTab;
use app::{App, Focus};
use event::AppEvent;
use pty::manager::PtyManager;
use ui::command_bar::CommandBar;
use ui::file_browser_pane::FileBrowserPane;
use ui::git_pane::GitPane;
use ui::layout::AppLayout;
use ui::prompt_dialog::PromptDialog;
use ui::pty_pane::PtyPane;
use ui::tab_bar::TabBar;

#[tokio::main]
async fn main() -> Result<()> {
    // Set panic hook to restore terminal before printing panic
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = tui::restore();
        original_hook(info);
    }));

    let result = run().await;

    // Always restore terminal
    tui::restore()?;

    result
}

async fn run() -> Result<()> {
    let mut terminal = tui::init().context("terminal init")?;
    let size = terminal.size().context("get terminal size")?;

    // Compute initial PTY size from layout (account for tab bar + command bar = 2 rows)
    let layout = AppLayout::new();
    let (_, content_area, _) = AppLayout::with_tab_and_command_bar(ratatui::layout::Rect::new(
        0, 0, size.width, size.height,
    ));
    let (pty_area, _) = layout.split(content_area);
    let (pty_cols, pty_rows) = AppLayout::pty_inner_size(pty_area);

    let (pty_manager, pty_reader) =
        PtyManager::spawn(pty_cols, pty_rows).context("spawn PTY")?;
    let mut app = App::new(pty_manager, pty_cols, pty_rows);

    // Event channel
    let (tx, mut rx) = mpsc::unbounded_channel::<AppEvent>();

    // Give App a clone of the sender for async git refresh
    app.event_tx = Some(tx.clone());

    // Initial git refresh (synchronous, before loop starts)
    app.refresh_git_sync();

    // Spawn PTY reader task (owns the reader half of the dup'd master fd)
    let tx_pty = tx.clone();
    tokio::spawn(async move {
        pty::manager::read_pty_loop(pty_reader, tx_pty).await;
    });

    // Spawn crossterm event reader
    let tx_input = tx.clone();
    tokio::spawn(async move {
        let mut reader = EventStream::new();
        while let Some(Ok(event)) = reader.next().await {
            let app_event = AppEvent::from(event);
            if tx_input.send(app_event).is_err() {
                break;
            }
        }
    });

    // Spawn tick timer
    let tx_tick = tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
        loop {
            interval.tick().await;
            if tx_tick.send(AppEvent::Tick).is_err() {
                break;
            }
        }
    });

    // Main loop: wait for events first, then batch, then draw
    while app.running {
        // Wait for at least one event
        if let Some(event) = rx.recv().await {
            app.handle_event(event).await?;
        } else {
            break;
        }

        // Drain all pending events before drawing (batch processing)
        while let Ok(event) = rx.try_recv() {
            app.handle_event(event).await?;
            if !app.running {
                break;
            }
        }

        // Draw once for all batched events
        if app.running {
            terminal.draw(|frame| {
                let size = frame.area();
                let (tab_area, content_area, cmd_area) =
                    AppLayout::with_tab_and_command_bar(size);

                // Store tab bar rect for mouse hit-testing
                app.tab_bar_rect = tab_area;

                // Render tab bar
                let tab_bar = TabBar::new(app.active_tab);
                tab_bar.render(tab_area, frame.buffer_mut());

                match app.active_tab {
                    ActiveTab::ClaudeCode => {
                        let (pty_area, git_area) = app.layout.split(content_area);
                        let (status_area, diff_area) = AppLayout::split_right(git_area);

                        // Store rects for mouse hit-testing and drag resize
                        app.main_area = content_area;
                        app.update_rects(pty_area, status_area, diff_area);

                        // Resize PTY if needed
                        app.resize_pty(pty_area);

                        // Render PTY pane
                        let pty_pane = PtyPane::new(&app.emulator, app.focus == Focus::Pty, &app.pty_selection);
                        pty_pane.render(pty_area, frame.buffer_mut());

                        // Render Git pane
                        let git_pane = GitPane {
                            files: &app.files,
                            diff: app.current_diff.as_ref(),
                            branch: &app.branch,
                            focus: app.focus,
                            status_state: &mut app.status_state,
                            diff_state: &app.diff_state,
                        };
                        git_pane.render(git_area, frame.buffer_mut());
                    }
                    ActiveTab::FileBrowser => {
                        // Ensure scroll offsets are correct for visible panels
                        let inner_height = content_area.height.saturating_sub(2) as usize;
                        app.file_browser.left.ensure_visible(inner_height);
                        app.file_browser.right.ensure_visible(inner_height);

                        let fb_pane = FileBrowserPane::new(&app.file_browser);
                        fb_pane.render(content_area, frame.buffer_mut());

                        // Clear pane rects so Claude Code mouse handling doesn't fire
                        app.main_area = content_area;
                        app.update_rects(
                            ratatui::layout::Rect::default(),
                            ratatui::layout::Rect::default(),
                            ratatui::layout::Rect::default(),
                        );
                    }
                }

                // Command bar
                let cmd_bar = CommandBar::new(
                    app.focus,
                    app.status_state.multi_select,
                    app.active_tab,
                );
                cmd_bar.render(cmd_area, frame.buffer_mut());

                // Prompt dialog (modal overlay)
                if app.prompt_state.visible {
                    let dialog = PromptDialog::new(&app.prompt_state);
                    dialog.render(content_area, frame.buffer_mut());
                }
            })?;
        }
    }

    Ok(())
}
