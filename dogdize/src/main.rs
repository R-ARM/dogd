use std::{
    env,
    process::{Command, Stdio, exit},
    thread,
    time::Duration,
    io::{Read, BufRead, BufReader},
};
use libdogd::{post_log, LogPriority};

fn push2dogd(stream: impl Read, name: &str, priority: LogPriority) {
    let mut writer = BufReader::new(stream);
    let mut buf = String::new();

    while let Ok(_) = writer.read_line(&mut buf) {
        if buf.is_empty() {
            continue;
        }
        post_log(&buf, name, priority);
        buf.clear();
    }
}

fn main() {
    let mut args = env::args_os().skip(1);
    let executable = args.next().expect("1st argument has to be the executable i gotta run");
    let name = executable.clone().into_string().unwrap();

    let stdin = Stdio::null();
    let stdout = Stdio::piped();
    let stderr = Stdio::piped();

    let mut child = Command::new(executable)
        .args(args)
        .stdin(stdin)
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
        .expect("Failed to execute our child");

    thread::scope(|s| {
        let child_stdout = child.stdout.take().unwrap();
        s.spawn(|| push2dogd(child_stdout, &name, LogPriority::Info));

        let child_stderr = child.stderr.take().unwrap();
        s.spawn(|| push2dogd(child_stderr, &name, LogPriority::Error));

        child.wait().unwrap();
        thread::sleep(Duration::from_millis(100));
        exit(0);
    });
}
