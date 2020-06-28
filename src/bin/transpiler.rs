use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::str::SplitWhitespace;

const FILENAME: &str = "output.asm";

#[derive(Debug)]
enum AsmLineError {
    UnknownError,
    UnknownOpcode(String),
    IncorrectNumberOfArguments,
}

#[derive(Debug)]
enum AsmLine {
    Label(String),
    Xor(String, String),
    Mov(String, String),
    Inc(String),
    Jmp(String),
}

impl AsmLine {
    fn args<'a>(
        parts: &'a mut SplitWhitespace,
        expected_count: usize,
    ) -> Result<Vec<&'a str>, AsmLineError> {
        let args = parts.take(2).collect::<Vec<&str>>();
        if args.len() == expected_count {
            Ok(args)
        } else {
            Err(AsmLineError::IncorrectNumberOfArguments)
        }
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
                "xorl" => {
                    return AsmLine::args(&mut parts, 2)
                        .and_then(|args| Ok(Self::Xor(args[0].to_string(), args[1].to_string())));
                }
                "movb" => {
                    return AsmLine::args(&mut parts, 2)
                        .and_then(|args| Ok(Self::Mov(args[0].to_string(), args[1].to_string())));
                }
                "incb" => {
                    return AsmLine::args(&mut parts, 1)
                        .and_then(|args| Ok(Self::Inc(args[0].to_string())));
                }
                "jmp" => {
                    return AsmLine::args(&mut parts, 1)
                        .and_then(|args| Ok(Self::Jmp(args[0].to_string())));
                }
                _ => return Err(AsmLineError::UnknownOpcode(opcode.to_string())),
            }
        }

        Err(AsmLineError::UnknownError)
    }
}

fn main() -> Result<(), std::io::Error> {
    let file = File::open(FILENAME)?;
    let file = BufReader::new(&file);

    let x: Vec<AsmLine> = file
        .lines()
        .skip(1)
        .enumerate()
        .map(|(num, l)| {
            print!("Line {:4}\t\t", num);
            l.unwrap()
        })
        .map(|s| {
            println!("{}", s);
            s.parse::<AsmLine>()
        })
        .map(|s| s.expect("Parse error"))
        .collect();

    println!("{:?}", x);

    Ok(())
}
