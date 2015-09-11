/**
 * THINGS THAT THIS NEEDS
 * Directional operators (<, >)
 * backgrounding
 * an environment
 * the sh language
 * more builtins
 * how to support less/man/vim etc
 */
extern crate copperline;

use std::process::*;
use copperline::Copperline;
use std::io::{stdout, Write, Read};

fn main() {
    let mut cl = Copperline::new();
    loop {
        match cl.read_line("$") {
            Ok(line) => {
                print!("{}", process_line(&line));
                stdout().flush().unwrap();
                cl.add_history(line);
            },
            _       => {},
        }
    }
}

fn process_line(line: &String) -> String {
    // list of processes to run, with input fed from output sequentially
    let cmds = line.trim().split('|').collect::<Vec<&str>>();
    let mut ret = String::new();
    if cmds.len() == 0 {
        return ret;
    }
    ret = process(
        String::from(cmds[0].trim())
            .split(' ')
            .collect::<Vec<&str>>());
    for i in 1..cmds.len() {
        let input = ret;
        ret = execute_p(
            String::from(cmds[i].trim())
                .split(' ')
                .collect::<Vec<&str>>(), Some(input));
    }
    ret
}

fn process(words: Vec<&str>) -> String {
    if words.len() == 0 {
        return String::from("");
    }
    execute(words)
}
    
fn execute(words: Vec<&str>) -> String {
    execute_p(words, None)
}

fn execute_p(words: Vec<&str>, pin: Option<String>) -> String {
    let mut command;    
    let mut input: Option<String>;

    let cwords = words.clone();
    let mut worditer = cwords.into_iter();
    match worditer.next() {
        // builtins parsed here
        Some("exit")  => exit(0),
        Some("cd")    => {
            if words.len() == 1 {
                chdir("/");
            }
            else {
                chdir(words[1]);
            }
            return String::from("");
        },
        _       => command = Command::new(words[0]),
    }
    for word in worditer {
        command.arg(word);
    }
    match pin {
        Some(inp) => {
            command.stdin(Stdio::piped());
            input = Some(inp);
        },
        None => {
            input = None;
        },
    };
    // all output is piped
    command.stdout(Stdio::piped());
    let mut child: Child; 
    match command.spawn() {
        Ok(c) => child = c,
        Err(e) => return format!("err: {}", e),
    };
    
    match input {
        Some(inp) => wait_child_inp(&mut child, inp),
        None => wait_child(&mut child),
    }
}

fn wait_child_inp(child: &mut Child, input: String) -> String {
    match child.stdin {
        Some(ref mut stdin) => stdin.write_all(input.as_bytes()).unwrap(),
        None => return String::from("failed to find child stdin"),
    };
    child.wait().unwrap();

    let mut buf: String = String::new();
    match child.stdout {
        Some(ref mut out) => {
            out.read_to_string(&mut buf).unwrap();
        },
        None => {},
    }
    buf
}

fn wait_child(child: &mut Child) -> String {
    child.wait().unwrap();

    let mut buf: String = String::new();
    match child.stdout {
        Some(ref mut out) => {
            out.read_to_string(&mut buf).unwrap();
        },
        None => {},
    }
    buf
}

fn chdir(path: &str) {
    use std::env::set_current_dir;
    use std::path::Path;
    set_current_dir(&Path::new(path)).unwrap_or_else(|e| {
        println!("{}", e);
    });
}
