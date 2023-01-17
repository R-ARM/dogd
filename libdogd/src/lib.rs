use serde::{Serialize, Deserialize};
use colored::Colorize;
use std::{
    io::Write,
    net::{Shutdown, TcpStream},
    time::{UNIX_EPOCH, SystemTime, Duration},
    env,
};

pub static LOG_INPUT_ADDR: &'static str = "127.0.0.1:4001";
pub static LOG_OUTPUT_ADDR: &'static str = "127.0.0.1:4002";

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum LogPriority {
    Critical,
    Error,
    Info,
    Debug,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogLine {
    pub line: String,
    pub prog_name: String,
    pub priority: LogPriority,
    pub time: Duration,
}

fn curr_program() -> String {
    let raw_name = std::env::args().next().unwrap_or("<unknown>".to_string());
    raw_name.split('/').last().unwrap().to_string()
}

pub fn format_log(line: LogLine) -> String {
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

pub fn log_critical(line: impl ToString) {
    post_log(line.to_string(), curr_program(), LogPriority::Critical)
}

pub fn log_error(line: impl ToString) {
    post_log(line.to_string(), curr_program(), LogPriority::Error)
}

pub fn log_info(line: impl ToString) {
    post_log(line.to_string(), curr_program(), LogPriority::Info)
}

pub fn log_debug(line: impl ToString) {
    post_log(line.to_string(), curr_program(), LogPriority::Debug)
}

pub fn log_rust_error(err: impl std::error::Error, description: impl ToString, priority: LogPriority) {
    let mut msg = Vec::new();
    msg.push(description.to_string());
    msg.push(err.to_string());
    while let Some(err) = err.source() {
        msg.push(err.to_string());
    }
    post_log(msg.into_iter().map(|v| v + "\n").collect::<String>(), curr_program(), priority);
}

pub fn post_log(line: impl ToString, prog_name: impl ToString, priority: LogPriority) {
    if let Err(e) = _post_log(line, prog_name, priority) {
        #[cfg(feature = "stdout")] {
            eprintln!("libdogd: Failed to post log message!");
            eprintln!("{}", e);
        }
    }
}

fn _post_log(line: impl ToString, prog_name: impl ToString, priority: LogPriority) -> Result<(), anyhow::Error> {
    let mut stream = TcpStream::connect(LOG_INPUT_ADDR)?;
    let pkg = LogLine {
        line: line.to_string(),
        prog_name: prog_name.to_string(),
        priority,
        time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
    };
    let string = toml::to_string(&pkg)?;
    stream.write_all(string.as_bytes())?;
    stream.shutdown(Shutdown::Both)?;
    if cfg!(feature = "stdout") || env::var("LIBDOGD_FORCE_STDOUT_LOG").is_ok() {
        println!("{}", format_log(pkg));
    }
    Ok(())
}
