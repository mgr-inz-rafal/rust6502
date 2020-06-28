#![feature(try_trait)]

use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::option::NoneError;
use std::str::FromStr;
use std::str::SplitWhitespace;

const FILENAME: &str = "output.asm";

#[derive(Debug)]
enum AsmLineError {
    UnknownError,
    UnknownOpcode(String),
    IncorrectNumberOfArguments,
    EmptyArgument,
    MalformedArgument,
}

impl From<NoneError> for AsmLineError {
    fn from(e: NoneError) -> Self {
        AsmLineError::MalformedArgument
    }
}

macro_rules! opcode2 {
    ($parts:expr, $opcode:path) => {
        return AsmLine::args(&mut $parts, 2).and_then(|args| {
            Ok($opcode(
                args[0].parse::<Arg>().unwrap(),
                args[1].parse::<Arg>().unwrap(),
            ))
        });
    };
}

macro_rules! opcode1 {
    ($parts:expr, $opcode:path) => {
        return AsmLine::args(&mut $parts, 1).and_then(|args| Ok($opcode(args[0].to_string())));
    };
}

#[derive(Debug)]
enum Arg {
    Literal(i32),
    Register(char),
    AbsoluteAddress(i32),
    _RelativeAddress(i32),
}

impl FromStr for Arg {
    type Err = AsmLineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(AsmLineError::EmptyArgument);
        }

        let mut i = s.chars();
        Ok(i.next().and_then(|c| match c {
            '$' => return Some(Self::Literal(
                {
                    let abc: String = i.filter(|c| *c != ',').collect();
                    abc.parse::<i32>().unwrap()
                }
            )),
            '%' => return Some(Self::Register('A')),
            '0'..='9' => return Some(Self::AbsoluteAddress(3)),
            _ => return None,
        })?)
    }
}

#[derive(Debug)]
enum AsmLine {
    Label(String),
    Xor(String, String),
    Mov(Arg, Arg),
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
                "movb" => opcode2!(parts, Self::Mov),
                // "xorl" => opcode2!(parts, Self::Xor),
                // "incb" => opcode1!(parts, Self::Inc),
                // "jmp" => opcode1!(parts, Self::Jmp),
                _ => return Err(AsmLineError::UnknownOpcode(opcode.to_string())),
            }
        }

        Err(AsmLineError::UnknownError)
    }
}

impl fmt::Display for AsmLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", "")
    }
}

fn main() -> Result<(), std::io::Error> {
    let file = File::open(FILENAME)?;
    let file = BufReader::new(&file);

    let input: Vec<AsmLine> = file
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

    input.into_iter().for_each(|l| println!("{:?}", l));

    Ok(())
}
