use std::str::ParseBoolError;

fn parse_bool_error() -> Result<bool, ParseBoolError> {
    "123123".parse()
}

fn main() {
    libdogd::log_rust_error(parse_bool_error().unwrap_err(), "Failed to parse a boolean", libdogd::LogPriority::Error);
}
