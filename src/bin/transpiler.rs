#![feature(try_trait)]

#[macro_use]
extern crate lazy_static;
use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::option::NoneError;
use std::str::{FromStr, SplitWhitespace};
use std::sync::Mutex;

lazy_static! {
    static ref VREGS: Mutex<HashSet<char>> = Mutex::new(HashSet::new());
}

const FILENAME: &str = "output.asm";

#[derive(Debug)]
enum AsmLineError {
    UnknownError,
    MutexError,
    UnknownOpcode(String),
    IncorrectNumberOfArguments,
    EmptyArgument,
    MalformedArgumentName(String),
    MalformedRegisterName(String),
}

impl From<NoneError> for AsmLineError {
    fn from(_: NoneError) -> Self {
        AsmLineError::UnknownError
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
    Accumulator,
    VirtualRegister(char),
    AbsoluteAddress(i32),
    _RelativeAddress(i32),
    Label(String),
}

impl Arg {
    fn register_from_name(name: &str) -> Result<char, AsmLineError> {
        match name {
            "eax" | "al" => Ok('A'),
            "ecx" | "cl" => Ok('C'),
            "edx" => Ok('D'),
            _ => Err(AsmLineError::MalformedRegisterName(name.to_string())),
        }
    }
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Label(s) => write!(f, "{}", s),
            Self::Accumulator => write!(f, "A"),
            _ => write!(f, "Unable to generate 6502 code for argument: {:?}", self),
        }
    }
}

impl PartialEq for Arg {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Accumulator, Self::Accumulator) => true,
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
                .and_then(|c| match c {
                    'A' => Ok(Self::Accumulator),
                    _ => VREGS
                        .lock()
                        .and_then(|mut vregs| {
                            vregs.insert(c);
                            Ok(Self::VirtualRegister(c))
                        })
                        .map_err(|_| AsmLineError::MutexError),
                }),
                '.' => Ok(Self::Label({
                    it.skip(1).filter(|c| *c != ',').collect::<String>()
                })),
                '0'..='9' => Ok(Self::AbsoluteAddress({
                    it.filter(|c| *c != ',')
                        .collect::<String>()
                        .parse::<i32>()
                        .unwrap()
                })),
                '$' => Ok(Self::Literal({
                    it.skip(1)
                        .filter(|c| *c != ',')
                        .collect::<String>()
                        .parse::<i32>()
                        .unwrap()
                })),
                _ => Err(AsmLineError::MalformedArgumentName(s.to_string())),
            }?)
        } else {
            return Err(AsmLineError::UnknownError);
        }
    }
}

#[derive(Debug)]
enum AsmLine {
    Label(String),
    Xor(Arg, Arg),
    Adc(Arg, Arg),
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
            return Ok(Self::Label(line[1..].to_string()));
        }

        let mut parts = line.split_whitespace();
        if let Some(opcode) = parts.next() {
            match opcode {
                "movb" | "movzbl" => opcode_with_2_args!(parts, Self::Mov),
                "xorl" => opcode_with_2_args!(parts, Self::Xor),
                "addb" => opcode_with_2_args!(parts, Self::Adc),
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
            Self::Adc(l, r) => match (l, r) {
                (Arg::AbsoluteAddress(a), Arg::Accumulator) => {
                    writeln!(f,
                        "\tCLC\n\
                         \tADC {}"
                         ,a)
                },
                _ => writeln!(f, "Unable to generate code for opcode 'ADC' with combination of arguments: '{:?}' and '{:?}'", l, r),
            },
            Self::Mov(l, r) => match (l, r) {
                (Arg::Literal(l), Arg::AbsoluteAddress(a)) => {
                    writeln!(f,
                        "\tPHA\n\
                         \tLDA #{literal}\n\
                         \tSTA {addr}\n\
                         \tPLA"
                         , literal=l, addr=a)
                },
                (Arg::Accumulator, Arg::AbsoluteAddress(a)) => {
                    writeln!(f, "\tSTA {}", a)
                },
                (Arg::AbsoluteAddress(a), Arg::Accumulator) => {
                    writeln!(f, "\tLDA {}", a)
                },
                (Arg::AbsoluteAddress(a), Arg::VirtualRegister(r)) => {
                    writeln!(f,
                        "\tPHA\n\
                         \tLDA {addr}\n\
                         \tSTA VREG_{reg}\n\
                         \tPLA"
                         , addr=a, reg=r)
                },
                (Arg::VirtualRegister(r), Arg::AbsoluteAddress(a)) => {
                    writeln!(f,
                        "\tPHA\n\
                         \tLDA VREG_{reg}\n\
                         \tSTA {addr}\n\
                         \tPLA"
                         , addr=a, reg=r)
                },
                _ => writeln!(f, "Unable to generate code for opcode 'MOV' with combination of arguments: '{:?}' and '{:?}'", l, r),
            },
            Self::Inc(a) => {
                match a {
                    Arg::Accumulator =>{
                        writeln!(f, "\tCLC\n\tADC #1")
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

    eprintln!("Parsing input file...");
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
    eprintln!("Parsing complete.");
    eprintln!();

    eprintln!("Generating 6502 code...");
    if let Err(e) = VREGS.lock().and_then(|vregs| {
        vregs
            .iter()
            .for_each(|reg| println!(".ZPVAR .BYTE VREG_{}", reg));
        Ok(())
    }) {
        eprintln!("ERROR: Can't generate virtual registers: {}", e);
    }
    println!("\tORG $2000");
    input.into_iter().for_each(|l| print!("{}", l));
    eprintln!("Code generation complete.");
    eprintln!();

    Ok(())
}
