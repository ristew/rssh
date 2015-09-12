/**
 * THINGS THAT THIS NEEDS
 * Directional operators (<)
 * backgrounding
 * an environment
 * the sh language
 * more builtins
 */
extern crate copperline;

use std::process::*;
use copperline::Copperline;
use std::io::{stdout, Write, Read};
use std::fs::File;

fn main() {
    let mut cl = Copperline::new();
    loop {
        match cl.read_line("$") {
            Ok(line) => {
                print!("{}", process_line(&line, false));
                stdout().flush().unwrap();
                cl.add_history(line);
            },
            _       => {},
        }
    }
}

macro_rules! words {
    ($s:expr) => (String::from($s.trim()).split(' ').collect::<Vec<&str>>());
}

//this is big and icky. it should be modularized.
fn process_line(line: &String, force_piped: bool) -> String {
    if line.contains(">") {
        let line_file = line.trim().split('>').collect::<Vec<&str>>();
        //probably need to use actual fds but that requires unsafe
        if line_file[1].trim() == "/dev/null" {
            process_line(&String::from(line_file[0]), true);
            return String::new();
        }
        let mut file = match File::open(line_file[1].trim()) {
            Ok(f) => f,
            Err(_) => {
                //TODO: Better account for errors here
                match File::create(line_file[1].trim()) {
                    Ok(f) => f,
                    Err(e) => return format!("{}", e),
                }
            },
        };
        let result = process_line(&String::from(line_file[0]), true);
        file.write_all(result.as_bytes()).unwrap();
        return String::new();
    }

    //list of processes to run, with input fed from output sequentially
    let cmds = line.trim().split('|').collect::<Vec<&str>>();
    let is_piped = force_piped || cmds.len() > 1;
    if cmds.len() == 0 {
        return String::new();
    }

    let mut ret = execute_p(words!(cmds[0]), None, is_piped);
    if cmds.len() == 1 {
        return ret;
    }

    for i in 1..cmds.len() - 1 {
        let input = ret;
        ret = execute_p(words!(cmds[i]), Some(input), true);
    }
    let input = ret;
    execute_p(words!(cmds[cmds.len() - 1]), Some(input), force_piped)
}

fn execute_p(words: Vec<&str>, pin: Option<String>, piped: bool) -> String {
    let mut command;    
    let mut input: Option<String>;

    let cwords = words.clone();
    let mut worditer = cwords.into_iter();
    match worditer.next() {
        //builtins parsed here
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

    if piped {
        command.stdout(Stdio::piped());
    }

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

    wait_child(child)
}

fn wait_child(child: &mut Child) -> String {
    //handle this better
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
