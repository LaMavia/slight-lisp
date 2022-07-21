use std::{
    fs::File,
    io::{self, BufRead, Read, Write},
};

use crate::{evaluator::Runtime, parse};

fn read() -> Option<String> {
    let mut line = String::new();
    let stdin = std::io::stdin();
    match stdin.lock().read_line(&mut line) {
        Ok(_) => Some(line),
        Err(_) => None,
    }
}

fn pr(result: String) {
    print!("{}\nsl> ", result);
    io::stdout().flush().unwrap();
}

fn ev(runtime: &mut Runtime, input: &String) -> bool {
    match input.as_str() {
        "exit" => return false,
        _ if input.starts_with("load ") => {
          let path = input[5..].to_string().trim().to_string();
          match File::open(&path).as_mut() {
            Err(err) => pr(format!("[error] {} ({})", err, path)),
            Ok(f) => {
                let mut buf = String::new();

                f.read_to_string(&mut buf).unwrap();

                return ev(runtime, &buf);
            }
        }},
        _ if !input.is_empty() => match runtime.eval(&parse!(input)) {
            Ok(r) => pr(format!("{}", r)),
            Err(err) => pr(format!("[\\e[1;91meerror\\e[0m] {err}")),
        },
        _ => pr("".to_string()),
    };

    true
}

pub fn run() {
    pr("".to_string());

    let mut runtime = Runtime::new();

    loop {
        let input = read().unwrap_or("".to_string());

        if !ev(&mut runtime, &input) {
            break;
        }
    }
}
