use std::collections::HashMap;
use std::io::Result;
use std::net::SocketAddr;
use std::{io::stdout, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::layout::Margin;

use ratatui::widgets::{
    List, ListDirection, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::{CrosstermBackend, Terminal},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::sync::mpsc::Sender;

use crate::shared::RoutingTableEntry;
use crate::{
    channel_events::{ChannelEvent, Commands},
    shared::Rx,
};

struct TUI {
    input: String,
    input_history: Vec<String>,
    input_history_index: usize,
    log_index: usize,
    log: Vec<String>,
    chat_room: Vec<String>,
    exit: bool,
    receiver: Rx,
    sender: Sender<ChannelEvent>,
    contacts: HashMap<SocketAddr, RoutingTableEntry>,
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
        "setnick" => {
            if words.len() < 2 {
                Commands::Unknown("Invalid number of arguments".to_string())
            } else {
                let name = words.get(1).unwrap_or(&"Morganite").to_string();
                Commands::SetOwnNick(name)
            }
        }
        "msg" => {
            if words.len() < 4 {
                Commands::Unknown("Invalid number of arguments".to_string())
            } else {
                let ip = words.get(1).unwrap_or(&"").to_string();
                let port = words.get(2).unwrap_or(&"").to_string();
                // Any message after the first 3 words is the message
                let msg = words[3..].join(" ");
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
        input_history: Vec::new(),
        input_history_index: 0,
        log_index: 0,
        log: Vec::new(),
        chat_room: Vec::new(),
        sender: fake_tx,
        exit: false,
        contacts: HashMap::new(),
    };

    // Create a timer that fires a tick every 3s
    let mut current_time = std::time::Instant::now();

    help_cmd(&mut tui);

    // Main Loop
    while !tui.exit {
        terminal.draw(|frame| {
            draw_ui(frame, &tui).unwrap();
        })?;

        // Send a contacts request every 3 seconds to keep the routing table updated
        if current_time.elapsed() > Duration::from_secs(3) {
            current_time = std::time::Instant::now();
            let _ = tui.sender.send(ChannelEvent::Command(Commands::Contacts));
        }

        // Check for received messages
        if let Ok(event) = tui.receiver.try_recv() {
            match event {
                ChannelEvent::Message(msg, addr) => {
                    tui.chat_room.push(format!("{}: {}", addr, msg));
                }
                ChannelEvent::Command(cmd) => {
                    tui.log.push(format!("Received command: {:?}", cmd));
                }
                ChannelEvent::CommandReceiver(tx) => {
                    tui.log.push("Received command receiver".to_string());
                    tui.sender = tx;
                }
                ChannelEvent::Contacts(contacts) => {
                    //tui.log.push(format!("Received contacts: {:?}", contacts));
                    tui.contacts = contacts;
                }
                ChannelEvent::Join(addr) => {
                    tui.chat_room.push(format!("New User joined @ {}", addr));
                }
                ChannelEvent::Leave(addr) => {
                    tui.chat_room.push(format!("User left @ {}", addr));
                }
                ChannelEvent::MessageToTUI(msg, name, addr) => {
                    tui.chat_room.push(format!("{}@{}: {}", name, addr, msg));
                }
                ChannelEvent::LogToTerminal(msg) => {
                    tui.log.push(msg);
                }
                ChannelEvent::Routing(_) => {
                    // Do nothing, spammy
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
                        KeyCode::Up => {
                            tui.input_history_index = tui.input_history_index.saturating_sub(1);
                            tui.input = tui
                                .input_history
                                .get(tui.input_history_index)
                                .unwrap_or(&"".to_string())
                                .to_string();
                        }
                        KeyCode::Down => {
                            tui.input_history_index = tui.input_history_index.saturating_add(1);
                            tui.input = tui
                                .input_history
                                .get(tui.input_history_index)
                                .unwrap_or(&"".to_string())
                                .to_string();
                        }
                        KeyCode::Left => {
                            if tui.log_index < tui.log.len() {
                                tui.log_index = tui.log_index.saturating_add(1);
                            }
                        }
                        KeyCode::Right => {
                            tui.log_index = tui.log_index.saturating_sub(1);
                        }
                        KeyCode::Enter => {
                            // Process input
                            let cmd = command_to_event(tui.input.as_str());
                            tui.input_history.push(tui.input.clone()); // Save the input to history
                            tui.input_history_index = tui.input_history.len(); // Reset the history index

                            // Prepare to exit if the command is quit
                            match cmd {
                                Commands::Quit => {
                                    tui.exit = true;
                                }
                                Commands::SetOwnNick(ref name) => {
                                    tui.chat_room.push(format!("Set own nick to: {}", name));
                                }
                                Commands::Message(ref addr, ref message) => {
                                    tui.chat_room.push(format!("You => {}: {}", addr, message));
                                }
                                _ => {}
                            }

                            match cmd {
                                Commands::Help => {
                                    help_cmd(&mut tui);
                                }
                                _ => {
                                    let response =
                                        tui.sender.send(ChannelEvent::Command(cmd.clone()));
                                    tui.log.push(format!(
                                        "Sending command: {:?} => {:#?}",
                                        cmd, response
                                    ));
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
                    // Do nothing
                }
            };
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn help_cmd(tui: &mut TUI) {
    tui.log.push(
        "Available commands: \n\
        =====================\n\
        quit => Quit client\n\
        help => Get this message again\n\
        contacts => Retrieve routing table (Also in the right block)\n\
        msg <IP> <port> <message> => Send a message to somebody\n\
        connect <IP> <port> => Connect to a new peer\n\
        setnick <name> => Set your own nickname\n\
        ↑ => Previous command\n\
        ↓ => Next command\n\
        ← => Go back in log\n\
        → => Go forward in log\
    "
        .to_string(),
    );
}

fn draw_ui(frame: &mut Frame, tui: &TUI) -> Result<()> {
    let root_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(10), Constraint::Percentage(90)])
        .split(frame.size());

    let top_inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(root_layout[1]);

    // Top Input Field
    let input = tui.input.clone();
    frame.render_widget(
        Paragraph::new(input).block(Block::new().borders(Borders::ALL).title("Input")),
        root_layout[0],
    );

    // Display the log
    // Only showcase the last 10 logs
    let mut log = tui.log.split_at(tui.log.len() - tui.log_index).0.to_vec();
    log.reverse();

    frame.render_widget(
        List::new(log)
            .block(Block::new().borders(Borders::ALL).title("Logs"))
            .direction(ListDirection::BottomToTop),
        top_inner_layout[0],
    );

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    let mut scrollbar_state =
        ScrollbarState::new(tui.log.len()).position(tui.log.len() - tui.log_index);

    frame.render_stateful_widget(
        scrollbar,
        top_inner_layout[0].inner(&Margin {
            // using an inner vertical margin of 1 unit makes the scrollbar inside the block
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );

    // Display the messages / join / leave
    // Display the log
    // Only showcase the last 10 logs
    let mut chat: Vec<String> = tui.chat_room.clone();
    chat.reverse();

    frame.render_widget(
        List::new(chat)
            .block(Block::new().borders(Borders::ALL).title("Chat"))
            .direction(ListDirection::BottomToTop),
        top_inner_layout[1],
    );

    // Display Routing Entries
    let mut rounting_entries =
        "Node Addr: | Hops | Via Addr:\n =============================\n".to_string();
    for (addr, entry) in tui.contacts.iter() {
        let entry = format!("{:?} | {:?} | {:?} \n", addr, entry.hop_count, entry.next);
        rounting_entries.push_str(&entry);
    }

    frame.render_widget(
        Paragraph::new(rounting_entries)
            .block(Block::new().borders(Borders::ALL).title("Routing Table"))
            .wrap(Wrap { trim: true }),
        top_inner_layout[2],
    );

    Ok(())
}
