use std::fs::File;
use std::io::{BufRead, BufReader};

const FILENAME: &str = "output.asm";

fn main() -> Result<(), std::io::Error> {
    let file = File::open(FILENAME)?;
    let file = BufReader::new(&file);

    file.lines().enumerate().for_each(|(line, text)| println!("{} -> {:?}", line, text));

    Ok(())
}