use std::io::{Write, stdin, stdout};

pub fn prompt(prompt: &str) -> String {
    let mut s = String::new();
    loop {
        let _ = stdout().write(prompt.as_bytes());
        let _ = stdout().flush();
        stdin().read_line(&mut s).expect("Did not enter a correct string");
        let s = s.trim();
        if s.is_empty() {
            continue;
        }
        return s.to_string();
    }
}
