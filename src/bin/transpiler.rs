use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::str::SplitWhitespace;

const FILENAME: &str = "output.asm";

#[derive(Debug)]
enum AsmLineError {
    UnknownError,
    UnknownOpcode(String),
    IncorrectArgs
}

#[derive(Debug)]
enum AsmLine {
    Label(String),
    Xor(String, String)
}

impl AsmLine {
    fn to_args<'a>(parts: &'a mut SplitWhitespace, expected_count: usize) -> Result<Vec<&'a str>, &'static str> {
       let args = parts.take(2).collect::<Vec<&str>>();
       if args.len() == expected_count { Ok(args) } else { Err("Incorrect number of arguments") }
    }
}

impl FromStr for AsmLine {
    type Err = AsmLineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let line = s.to_string();

        if line.starts_with('.') {
            return Ok(Self::Label(line));
        }

        let mut parts = line.split_whitespace();
        if let Some(opcode) = parts.next() {
            match opcode {
                "xorl" => if let Ok(args) = AsmLine::to_args(&mut parts, 2) {
                    return Ok(Self::Xor(args[0].to_string(), args[1].to_string()));
                } else {
                    return Err(AsmLineError::IncorrectArgs)
                },
                // "movb" => {},
                // "incb" => {},
                // "jmp" => {},
                _ => return Err(AsmLineError::UnknownOpcode(opcode.to_string()))
            }
        }

        Err(AsmLineError::UnknownError)
    }
}

fn main() -> Result<(), std::io::Error> {
    let file = File::open(FILENAME)?;
    let file = BufReader::new(&file);

    for (num, line) in file.lines().skip(1).enumerate() {
        if let Ok(line) = line {
            println!("Parsing line {}: {:?}", num, line);
            let line = line.parse::<AsmLine>();

            println!("   {:?}", line);
            println!();
        }
    }

    Ok(())
}