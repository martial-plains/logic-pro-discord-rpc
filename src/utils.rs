use std::process::Command;

pub fn shell_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        if c == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}

pub fn run_cmd(cmd: &str) -> String {
    let output = Command::new("sh").arg("-c").arg(cmd).output().unwrap();
    String::from_utf8(output.stdout).unwrap()
}
