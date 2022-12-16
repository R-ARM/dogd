use libdogd::{LogPriority, post_log, log_critical, log_error, log_info, log_debug};

fn main() {
    post_log("Hello World!".to_string(), "hello".to_string(), LogPriority::Info).unwrap();

    log_error("this is an error").unwrap();
    log_info("is is an info message").unwrap();
    log_debug("this is a debug message").unwrap();
    log_critical("note that this will display an error message!").unwrap();
}
