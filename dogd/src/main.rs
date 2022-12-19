use anyhow::Result;
use colored::Colorize;
use multiqueue::{broadcast_queue, BroadcastSender, BroadcastReceiver};
use libdogd::{LogLine, LOG_INPUT_ADDR, LOG_OUTPUT_ADDR, LogPriority, log_error};
use std::{
    net::{TcpListener, TcpStream},
    io::{Read, Write},
    fs::File,
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

fn listen_for_log(tx: BroadcastSender<String>) -> Result<()> {
    let listener = TcpListener::bind(LOG_INPUT_ADDR)?;

    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        let mut packet = String::new();

        if stream.read_to_string(&mut packet).is_err() {
            continue;
        }

        let Ok(line) = toml::from_str::<LogLine>(&packet) else { continue };
        tx.try_send(format_log(line))?;
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream, rx: BroadcastReceiver<String>) -> Result<()> {
    loop {
        let line = rx.recv()?;
        stream.write_all(line.as_bytes())?;
    }
}

fn push_logs(rx: BroadcastReceiver<String>) -> Result<()> {
    let listener = TcpListener::bind(LOG_OUTPUT_ADDR)?;

    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let recv = rx.add_stream();
        thread::spawn(move || handle_client(stream, recv));
    }
    Ok(())
}

fn print_logs(rx: BroadcastReceiver<String>) -> Result<()> {
    while let Ok(line) = rx.recv() {
        print!("{}", line);
    }
    Ok(())
}

static LOG_PATH: &'static str = "/var/log/dogd";
fn save_logs(rx: BroadcastReceiver<String>) {
    let Ok(mut file) = File::create(LOG_PATH) else {
        log_error("Failed to open or create log file");
        return;
    };
    while let Ok(line) = rx.recv() {
        if file.write_all(line.as_bytes()).is_err() {
            return;
        }
    }
}

fn main() -> Result<()> {
    let (tx, rx) = broadcast_queue(512);

    let rx2 = rx.add_stream();
    let rx3 = rx.add_stream();
    let log_saver_thread = thread::spawn(move || save_logs(rx3));
    let log_printer_thread = thread::spawn(move || print_logs(rx2));
    let pusher_thread = thread::spawn(move || push_logs(rx));
    let listener_thread = thread::spawn(move || listen_for_log(tx));

    listener_thread.join().unwrap()?;
    log_saver_thread.join().unwrap();
    log_printer_thread.join().unwrap()?;
    pusher_thread.join().unwrap()?;

    Ok(())
}
