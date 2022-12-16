use libdogd::{LogPriority, post_log};

fn main() {
    post_log("Hello World!".to_string(), "hello".to_string(), LogPriority::Info).unwrap();
}
