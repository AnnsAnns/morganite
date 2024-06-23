use std::{io::{stdout, Stdout}, sync::{mpsc::Receiver, Arc}};
use std::io::{Result};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Direction, Layout}, prelude::{CrosstermBackend, Stylize, Terminal}, widgets::{Block, Borders, Paragraph}, Frame
};
use tokio::sync::Mutex;

use crate::{channel_events::ChannelEvent, shared::{Rx, Shared, Tx}};

struct TUI {
    input: String,
    log: Vec<String>,
    exit: bool,
    receiver: Rx,
    sender: Tx,
}

pub fn tui(receiver: Rx, sender: Tx) -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut tui = TUI {
        receiver,
        sender,
        input: String::new(),
        log: Vec::new(),
        exit: false,
    };

    // Main Loop
    loop {
        terminal.draw(|frame| {draw_ui(frame, &tui);})?;

        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Char(c) => {
                        if c == 'q' {
                            break;
                        }

                        tui.input.push(c);
                    }
                    KeyCode::Enter => {
                        // Process input
                        if tui.input == "quit" {
                            break;
                        }
                        tui.input.clear();
                    }
                    KeyCode::Backspace => {
                        tui.input.pop();
                    }
                    _ => {}
                }
            }
            _ => {tui.log.push("Unknown event".to_string());}
        };     
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn draw_ui(frame: &mut Frame, tui: &TUI) -> Result<()> {
    let root_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints(vec![
        Constraint::Percentage(10),
        Constraint::Percentage(90),
    ])
    .split(frame.size());

    let top_inner_layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints(vec![
        Constraint::Percentage(75),
        Constraint::Percentage(25),
    ])
    .split(root_layout[1]);

    let input = format!("Input: {}", tui.input);
    frame.render_widget(
        Paragraph::new(input)
            .block(Block::new().borders(Borders::ALL)),
            root_layout[0]);

    let peers = "Peers: \n";
    frame.render_widget(
        Paragraph::new(peers)
            .block(Block::new().borders(Borders::ALL)),
            top_inner_layout[1]);


    let log = tui.log.join("Input: \n");
    frame.render_widget(
        Paragraph::new(format!("Logs:\n {}", log))
            .block(Block::new().borders(Borders::ALL)),
            top_inner_layout[0]);
    Ok(())
}