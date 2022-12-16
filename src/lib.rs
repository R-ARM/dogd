use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::{
    io::Write,
    net::{Shutdown, TcpStream},
    time::{UNIX_EPOCH, SystemTime, Duration},
};

pub static LOG_INPUT_ADDR: &'static str = "127.0.0.1:4001";

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum LogPriority {
    Critical,
    Error,
    Info,
    Debug,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogLine {
    line: String,
    prog_name: String,
    priority: LogPriority,
    time: Duration,
}

fn curr_program() -> String {
    let raw_name = std::env::args().next().unwrap_or("<unknown>".to_string());
    raw_name.split('/').last().unwrap().to_string()
}

pub fn log_critical(line: impl ToString) -> Result<()> {
    post_log(line.to_string(), curr_program(), LogPriority::Critical)
}

pub fn log_error(line: impl ToString) -> Result<()> {
    post_log(line.to_string(), curr_program(), LogPriority::Error)
}

pub fn log_info(line: impl ToString) -> Result<()> {
    post_log(line.to_string(), curr_program(), LogPriority::Info)
}

pub fn log_debug(line: impl ToString) -> Result<()> {
    post_log(line.to_string(), curr_program(), LogPriority::Debug)
}

pub fn post_log(line: impl ToString, prog_name: impl ToString, priority: LogPriority) -> Result<()> {
    let mut stream = TcpStream::connect(LOG_INPUT_ADDR)?;
    let pkg = LogLine {
        line: line.to_string(),
        prog_name: prog_name.to_string(),
        priority,
        time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
    };
    stream.write_all(toml::to_string(&pkg)?.as_bytes())?;
    stream.shutdown(Shutdown::Both)?;
    Ok(())
}
