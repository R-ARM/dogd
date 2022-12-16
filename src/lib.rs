use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::{
    io::Write,
    net::{Shutdown, TcpStream},
    time::{UNIX_EPOCH, SystemTime, Duration},
};

pub static LOG_INPUT_ADDR: &'static str = "127.0.0.1:4001";

#[derive(Serialize, Deserialize, Debug)]
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

pub fn post_log(line: String, prog_name: String, priority: LogPriority) -> Result<()> {
    let mut stream = TcpStream::connect(LOG_INPUT_ADDR)?;
    let pkg = LogLine {
        line,
        prog_name,
        priority,
        time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
    };
    stream.write_all(toml::to_string(&pkg)?.as_bytes())?;
    stream.shutdown(Shutdown::Both)?;
    Ok(())
}
