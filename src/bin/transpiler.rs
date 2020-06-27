use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::str::SplitWhitespace;

const FILENAME: &str = "output.asm";

#[derive(Debug)]
enum AsmLineError {
    UnknownError,
    UnknownOpcode(String)
}

#[derive(Debug)]
enum AsmLine {
    Label(String),
    Xor(String, String)
}

impl AsmLine {
    fn to_params<'a>(parts: &'a mut SplitWhitespace) -> Result<Vec<&'a str>, String> {
       Ok(parts.take(2).collect::<Vec<&str>>())
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
                "xorl" => if let Ok(x) = AsmLine::to_params(&mut parts) {
                    return Ok(Self::Xor(x[0].to_string(), x[1].to_string()));
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