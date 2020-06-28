#![feature(try_trait)]

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
    fn from(_: NoneError) -> Self {
        AsmLineError::MalformedArgument
    }
}

macro_rules! opcode_with_2_args {
    ($parts:expr, $opcode:path) => {
        return AsmLine::args(&mut $parts, 2).and_then(|args| {
            Ok($opcode(
                args[0].parse::<Arg>().unwrap(),
                args[1].parse::<Arg>().unwrap(),
            ))
        });
    };
}

macro_rules! opcode_with_1_arg {
    ($parts:expr, $opcode:path) => {
        return AsmLine::args(&mut $parts, 1)
            .and_then(|args| Ok($opcode(args[0].parse::<Arg>().unwrap())));
    };
}

#[derive(Debug)]
enum Arg {
    Literal(i32),
    Register(char),
    AbsoluteAddress(i32),
    _RelativeAddress(i32),
    Label(String),
}

impl Arg {
    fn register_from_name(name: &str) -> Option<char> {
        match name {
            "eax" | "al" => Some('A'),
            _ => None,
        }
    }
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Label(s) => write!(f, "{}", s),
            Self::Register(c) => write!(f, "{}", c),
            _ => write!(f, "Unable to generate 6502 code for argument: {:?}", self),
        }
    }
}

impl PartialEq for Arg {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Register(r1), Self::Register(r2)) => r1 == r2,
            _ => false,
        }
    }
}

impl FromStr for Arg {
    type Err = AsmLineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(AsmLineError::EmptyArgument);
        }

        let mut it = s.chars().peekable();
        if let Some(c) = it.peek() {
            Ok(match c {
                '%' => Arg::register_from_name(
                    &it.skip(1)
                        .filter(|c| !vec![',', '%'].contains(c))
                        .collect::<String>(),
                )
                .and_then(|c| Some(Self::Register(c))),
                '.' => Some(Self::Label({
                    it.filter(|c| *c != ',').collect::<String>()
                })),
                '0'..='9' => Some(Self::AbsoluteAddress({
                    it.filter(|c| *c != ',')
                        .collect::<String>()
                        .parse::<i32>()
                        .unwrap()
                })),
                '$' => Some(Self::Literal({
                    it.skip(1)
                        .filter(|c| *c != ',')
                        .collect::<String>()
                        .parse::<i32>()
                        .unwrap()
                })),
                _ => None,
            }?)
        } else {
            return Err(AsmLineError::MalformedArgument);
        }
    }
}

#[derive(Debug)]
enum AsmLine {
    Label(String),
    Xor(Arg, Arg),
    Mov(Arg, Arg),
    Inc(Arg),
    Jmp(Arg),
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
                "movb" => opcode_with_2_args!(parts, Self::Mov),
                "xorl" => opcode_with_2_args!(parts, Self::Xor),
                "incb" => opcode_with_1_arg!(parts, Self::Inc),
                "jmp" => opcode_with_1_arg!(parts, Self::Jmp),
                _ => return Err(AsmLineError::UnknownOpcode(opcode.to_string())),
            }
        }

        Err(AsmLineError::UnknownError)
    }
}

impl fmt::Display for AsmLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Label(l) => writeln!(f, "{}", l),
            Self::Jmp(l) => writeln!(f, "\tJMP {}", l),
            Self::Xor(l, r) if l == r => writeln!(f, "\tLD{} #0", l),
            Self::Mov(l, r) => match (l, r) {
                (Arg::Literal(l), Arg::AbsoluteAddress(a)) => {
                    writeln!(f, "\tPHA")
                        .and_then(|_| writeln!(f, "\tLDA #{}", l))
                        .and_then(|_| writeln!(f, "\tSTA {}", a))
                        .and_then(|_| writeln!(f, "\tPLA"))
                },
                (Arg::Register(r), Arg::AbsoluteAddress(a)) => {
                    writeln!(f, "\tST{} {}", r, a)
                }
                _ => writeln!(f, "Unable to generate code for opcode 'MOV' with combination of arguments: '{:?}' and '{:?}'", l, r),
            },
            Self::Inc(a) => {
                match a {
                    Arg::Register(r) if *r == 'A' => {
                        writeln!(f, "\tCLC").and_then(|_| writeln!(f, "\tADC #1"))

                    }
                    _ => writeln!(f, "Unable to generate code for opcode 'INC' with argument: '{:?}'", a),
                }
            }
            _ => writeln!(f, "Unable to generate 6502 code for line: {:?}", self),
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    let file = File::open(FILENAME)?;
    let file = BufReader::new(&file);

    println!("Parsing input file...");
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
    println!("Parsing complete.");
    println!();

    println!("Generating 6502 code...");
    input.into_iter().for_each(|l| print!("{}", l));
    println!("Code generation complete.");
    println!();

    Ok(())
}
