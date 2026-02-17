#![allow(dead_code)]

mod action;
mod app;
mod event;
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

use app::{App, Focus};
use event::AppEvent;
use pty::manager::PtyManager;
use ui::command_bar::CommandBar;
use ui::git_pane::GitPane;
use ui::layout::AppLayout;
use ui::prompt_dialog::PromptDialog;
use ui::pty_pane::PtyPane;

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

    // Compute initial PTY size from layout
    let layout = AppLayout::new();
    let (main_area, _) = AppLayout::with_command_bar(ratatui::layout::Rect::new(
        0, 0, size.width, size.height,
    ));
    let (pty_area, _) = layout.split(main_area);
    let (pty_cols, pty_rows) = AppLayout::pty_inner_size(pty_area);

    let (pty_manager, pty_reader) =
        PtyManager::spawn(pty_cols, pty_rows).context("spawn PTY")?;
    let mut app = App::new(pty_manager, pty_cols, pty_rows);

    // Initial git refresh
    app.refresh_git();

    // Event channel
    let (tx, mut rx) = mpsc::unbounded_channel::<AppEvent>();

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

    // Main loop
    while app.running {
        // Draw
        terminal.draw(|frame| {
            let size = frame.area();
            let (main_area, cmd_area) = AppLayout::with_command_bar(size);
            let (pty_area, git_area) = app.layout.split(main_area);
            let (status_area, diff_area) = AppLayout::split_right(git_area);

            // Store rects for mouse hit-testing and drag resize
            app.main_area = main_area;
            app.update_rects(pty_area, status_area, diff_area);

            // Resize PTY if needed
            app.resize_pty(pty_area);

            // Render PTY pane
            let pty_pane = PtyPane::new(&app.emulator, app.focus == Focus::Pty);
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

            // Command bar
            let cmd_bar = CommandBar::new(app.focus, app.status_state.multi_select);
            cmd_bar.render(cmd_area, frame.buffer_mut());

            // Prompt dialog (modal overlay)
            if app.prompt_state.visible {
                let dialog = PromptDialog::new(&app.prompt_state);
                dialog.render(main_area, frame.buffer_mut());
            }
        })?;

        // Handle events (non-blocking drain)
        while let Ok(event) = rx.try_recv() {
            app.handle_event(event).await?;
        }

        // Wait for next event
        if app.running {
            if let Some(event) = rx.recv().await {
                app.handle_event(event).await?;
            }
        }
    }

    Ok(())
}
