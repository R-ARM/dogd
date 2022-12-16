use anyhow::Result;
use libdogd::{LogLine, LOG_INPUT_ADDR};
use std::{
    net::{TcpListener, TcpStream},
    io::Read,
};

fn main() -> Result<()> {

    let listener = TcpListener::bind(LOG_INPUT_ADDR)?;

    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        let mut packet = String::new();

        if stream.read_to_string(&mut packet).is_err() {
            continue;
        }

        let Ok(line) = toml::from_str::<LogLine>(&packet) else { continue };
        println!("{:#?}", line);
    }

    Ok(())
}
