use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::str::SplitWhitespace;

const FILENAME: &str = "output.asm";

#[derive(Debug)]
enum AsmLineError {
    UnknownError,
    UnknownOpcode(String),
    IncorrectArgs,
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
    ) -> Result<Vec<&'a str>, &'static str> {
        let args = parts.take(2).collect::<Vec<&str>>();
        if args.len() == expected_count {
            Ok(args)
        } else {
            Err("Incorrect number of arguments")
        }
    }
}

macro_rules! generate_opcode_2args {
    ($parts:expr, $opcode:path) => {
        if let Ok(args) = AsmLine::args(&mut $parts, 2) {
            return Ok($opcode(args[0].to_string(), args[1].to_string()));
        } else {
            return Err(AsmLineError::IncorrectArgs);
        }
    };
}

macro_rules! generate_opcode_1arg {
    ($parts:expr, $opcode:path) => {
        if let Ok(args) = AsmLine::args(&mut $parts, 1) {
            return Ok($opcode(args[0].to_string()));
        } else {
            return Err(AsmLineError::IncorrectArgs);
        }
    };
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
                "xorl" => generate_opcode_2args!(parts, Self::Xor),
                "movb" => generate_opcode_2args!(parts, Self::Mov),
                "incb" => generate_opcode_1arg!(parts, Self::Inc),
                "jmp" => generate_opcode_1arg!(parts, Self::Jmp),
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
        .map(|l| l.unwrap())
        .map(|s| s.parse::<AsmLine>())
        .map(|s| s.unwrap())
        .collect();

    println!("{:?}", x);

    Ok(())
}
