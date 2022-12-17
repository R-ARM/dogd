use anyhow::Result;
use colored::Colorize;
use crossbeam_channel::{bounded, Sender, Receiver};
use libdogd::{LogLine, LOG_INPUT_ADDR, LOG_OUTPUT_ADDR, LogPriority};
use std::{
    net::{TcpListener, TcpStream},
    io::{Read, Write},
    thread,
};

fn format_log(line: LogLine) -> String {
    let lines = line.line.trim()
        .split('\n')
        .collect::<Vec<&str>>();

    let level = match line.priority {
        LogPriority::Debug => "D".bright_black(),
        LogPriority::Info => "I".normal(),
        LogPriority::Error => "E".red(),
        LogPriority::Critical => "C".on_white().red(),
    };

    let mut buf = Vec::new();
    for this_line in lines {
        buf.push(format!("{}({}) {}\n", line.prog_name, level, this_line));
    }
    buf.into_iter().collect()
}

fn listen_for_log(tx: Sender<String>) -> Result<()> {
    let listener = TcpListener::bind(LOG_INPUT_ADDR)?;

    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        let mut packet = String::new();

        if stream.read_to_string(&mut packet).is_err() {
            continue;
        }

        let Ok(line) = toml::from_str::<LogLine>(&packet) else { continue };
        tx.send(format_log(line))?;
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream, rx: Receiver<String>) -> Result<()> {
    loop {
        let line = rx.recv()?;
        stream.write_all(line.as_bytes())?;
    }
}

fn push_logs(rx: Receiver<String>) -> Result<()> {
    let listener = TcpListener::bind(LOG_OUTPUT_ADDR)?;

    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let rx = rx.clone();
        thread::spawn(move || handle_client(stream, rx));
    }
    Ok(())
}

fn main() -> Result<()> {
    let (tx, rx) = bounded(512);

    let listener_thread = thread::spawn(move || listen_for_log(tx));
    let pusher_thread = thread::spawn(move || push_logs(rx));

    listener_thread.join().unwrap()?;
    pusher_thread.join().unwrap()?;

    Ok(())
}
