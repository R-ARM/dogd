use serde::{Serialize, Deserialize};
use std::{
    io::Write,
    net::{Shutdown, TcpStream},
    time::{UNIX_EPOCH, SystemTime, Duration},
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

pub fn post_log(line: impl ToString, prog_name: impl ToString, priority: LogPriority) {
    if let Err(e) = _post_log(line, prog_name, priority) {
        eprintln!("libdogd: Failed to post log message!");
        eprintln!("{}", e);
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
    Ok(())
}
