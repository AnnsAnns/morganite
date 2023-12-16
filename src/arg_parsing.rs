use std::{env::args, process::exit};

use log::warn;

pub fn help() {
    warn!("Usage: ./routingtable <port> <name (max 3 chars)>");
}

pub fn parse_port() -> String {
    match args().nth(1) {
        Some(port) => port,
        None => {
            help();
            exit(1);
        }
    }
}

pub fn parse_name() -> String {
    match args().nth(2) {
        Some(mut name) => {
            if name.len() > 3 {
                warn!("Name is too long, truncating to 3 characters");
                name.truncate(3);
                name
            } else {
                name
            }
        }
        None => {
            help();
            exit(1)
        }
    }
}