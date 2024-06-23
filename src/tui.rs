use std::io::Result;
use std::net::SocketAddr;
use std::{
    io::{stdout, Stdout},
    sync::{mpsc::Receiver, Arc},
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::widgets::{List, ListDirection};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::{CrosstermBackend, Stylize, Terminal},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::sync::mpsc::Sender;
use tokio::sync::{mpsc, Mutex};

use crate::{
    channel_events::{ChannelEvent, Commands},
    shared::{Rx, Shared, Tx},
};

struct TUI {
    input: String,
    log: Vec<String>,
    exit: bool,
    receiver: Rx,
    sender: Sender<ChannelEvent>,
}

fn string_to_socketaddr(ip: &str, port: &str) -> Option<SocketAddr> {
    let addr = format!("{}:{}", ip, port);
    match addr.parse::<SocketAddr>() {
        Ok(socketaddr) => Some(socketaddr),
        Err(_) => None,
    }
}

fn command_to_event(cmd: &str) -> Commands {
    let words = cmd.split(" ").collect::<Vec<&str>>();

    match words.get(0).unwrap_or(&"").to_owned() {
        "quit" => Commands::Quit,
        "help" => Commands::Help,
        "contacts" => Commands::Contacts,
        "msg" => {
            if words.len() < 4 {
                Commands::Unknown("Invalid number of arguments".to_string())
            } else {
                let ip = words.get(1).unwrap_or(&"").to_string();
                let port = words.get(2).unwrap_or(&"").to_string();
                let msg = words.get(3).unwrap_or(&"").to_string();
                let addr = match string_to_socketaddr(&ip, &port) {
                    Some(socketaddr) => socketaddr,
                    None => return Commands::Unknown("Invalid IP or Port".to_string()),
                };
                Commands::Message(addr, msg.to_string())
            }
        }
        "connect" => {
            if words.len() < 3 {
                Commands::Unknown("Invalid number of arguments".to_string())
            } else {
                let ip = words.get(1).unwrap_or(&"").to_string();
                let port = words.get(2).unwrap_or(&"").to_string();
                match string_to_socketaddr(&ip, &port) {
                    Some(socketaddr) => Commands::Connect(socketaddr),
                    None => Commands::Unknown("Invalid IP or Port".to_string()),
                }
            }
        }
        _ => Commands::Unknown(cmd.to_string()),
    }
}

pub fn tui(receiver: Rx) -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // Not used, just to satisfy the compiler
    let (fake_tx, _) = std::sync::mpsc::channel::<ChannelEvent>();

    let mut tui = TUI {
        receiver,
        input: String::new(),
        log: Vec::new(),
        sender: fake_tx,
        exit: false,
    };

    // Main Loop
    while !tui.exit {
        terminal.draw(|frame| {
            draw_ui(frame, &tui);
        })?;

        // Check for received messages
        if let Ok(event) = tui.receiver.try_recv() {
            match event {
                ChannelEvent::Message(msg, addr) => {
                    tui.log
                        .push(format!("Received message from {}: {}", addr, msg));
                }
                ChannelEvent::Command(cmd) => {
                    tui.log.push(format!("Received command: {:?}", cmd));
                }
                ChannelEvent::CommandReceiver(tx) => {
                    tui.log.push("Received command receiver".to_string());
                    tui.sender = tx;
                }
                ChannelEvent::Contacts(contacts) => {
                    tui.log.push(format!("Received contacts: {:?}", contacts));
                }
                ChannelEvent::Join(addr) => {
                    tui.log.push(format!("New User joined @ {}", addr));
                }
                ChannelEvent::Leave(addr) => {
                    tui.log.push(format!("User left @ {}", addr));
                }
                _ => tui.log.push(format!("Received unknown event: {:?}", event)),
            }
        }

        // Get inputs
        if let Ok(true) = event::poll(Duration::from_millis(300)) {
            match event::read()? {
                // it's important to check that the event is a key press event as
                // crossterm also emits key release and repeat events on Windows.
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Char(c) => {
                            tui.input.push(c);
                        }
                        KeyCode::Enter => {
                            // Process input
                            let cmd = command_to_event(tui.input.as_str());

                            // Prepare to exit if the command is quit
                            if cmd == Commands::Quit {
                                tui.exit = true;
                            }

                            match cmd {
                                Commands::Help => {
                                    tui.log.push(
                                        "Available commands: \n\
                                    quit\n\
                                    help\n\
                                    contacts\n\
                                    msg <IP> <port> <message>\n\
                                    connect <IP> <port>\
                                    "
                                        .to_string(),
                                    );
                                }
                                _ => {
                                    let response = tui.sender.send(ChannelEvent::Command(cmd.clone()));
                                    tui.log.push(format!("Sending command: {:?} => {:#?}", cmd, response));
                                }
                            }

                            tui.input.clear();
                        }
                        KeyCode::Backspace => {
                            tui.input.pop();
                        }
                        _ => {}
                    }
                }
                _ => {
                    tui.log.push("Unknown event".to_string());
                }
            };
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn draw_ui(frame: &mut Frame, tui: &TUI) -> Result<()> {
    let root_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(10), Constraint::Percentage(90)])
        .split(frame.size());

    let top_inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(root_layout[1]);

    let input = format!("Input: {}", tui.input);
    frame.render_widget(
        Paragraph::new(input).block(Block::new().borders(Borders::ALL)),
        root_layout[0],
    );

    let peers = "Peers: \n";
    frame.render_widget(
        Paragraph::new(peers).block(Block::new().borders(Borders::ALL)),
        top_inner_layout[1],
    );

    // Only showcase the last 10 logs
    let mut log = tui.log.clone();
    log.reverse();

    frame.render_widget(
        List::new(log).block(Block::new().borders(Borders::ALL)).direction(ListDirection::BottomToTop),
        top_inner_layout[0],
    );
    Ok(())
}
