use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

const FILENAME: &str = "output.asm";

#[derive(Debug)]
enum AsmLineError {
    UnknownError
}

#[derive(Debug)]
enum AsmLine {
    Label(String)
}

impl FromStr for AsmLine {
    type Err = AsmLineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let line = s.to_string();

        if line.starts_with('.') {
            return Ok(Self::Label(line));
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