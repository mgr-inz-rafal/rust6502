#![feature(try_trait)]

use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::option::NoneError;
use std::str::FromStr;

const FILENAME: &str = "output.asm";

#[derive(Debug)]
enum AsmLineError {
    UnknownError,
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
        return AsmLine::args($parts, 2).and_then(|args| {
            Ok($opcode(
                args[0].parse::<Arg>().unwrap(),
                args[1].parse::<Arg>().unwrap(),
            ))
        });
    };
}

macro_rules! opcode_with_1_arg {
    ($parts:expr, $opcode:path) => {
        return AsmLine::args($parts, 1)
            .and_then(|args| Ok($opcode(args[0].parse::<Arg>().unwrap())));
    };
}

#[derive(Debug)]
enum Arg {
    Literal(i32),
    Accumulator,
    VirtualRegister(char),
    AbsoluteAddress(i32),
    SumAddress(char, char),
    _RelativeAddress(i32),
    Label(String),
}

impl Arg {
    fn register_from_name(name: &str) -> Result<char, AsmLineError> {
        match name {
            "eax" | "al" => Ok('A'),
            "ecx" | "cl" => Ok('C'),
            "edx" | "dl" => Ok('D'),
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
                    _ => Ok(Self::VirtualRegister(c)),
                }),
                '.' => Ok(Self::Label({
                    it.skip(1).filter(|c| *c != ',').collect::<String>()
                })),
                '(' => {
                    let args: String = it.collect();
                    let args = args.trim_end_matches(")");
                    let args = args.trim_start_matches("(");
                    let args: Vec<String> = args.split(",").map(ToString::to_string).collect();

                    // TODO: Simplification: edx => D, ecx => C, etc.
                    Ok(Self::SumAddress(
                        args[0].chars().skip(2).next().unwrap(),
                        args[1].chars().skip(2).next().unwrap(),
                    ))
                }
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
    MovZ(Arg, Arg),
    Inc(Arg),
    Dec(Arg),
    Jmp(Arg),
}

impl AsmLine {
    fn args<'a, I>(parts: I, expected_count: usize) -> Result<Vec<String>, AsmLineError>
    where
        I: IntoIterator<Item = &'a String>,
    {
        let mut args: Vec<String> = vec![];

        let mut i = parts.into_iter();
        for _ in 0..2 {
            for first in i.next() {
                if first.starts_with("(") {
                    i.next()
                        .and_then(|second| Some(args.push(format!("{}{}", first, second))));
                } else {
                    args.push(first.to_owned());
                }
            }
        }

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

        let parts: Vec<String> = line.split_whitespace().map(ToString::to_string).collect();

        let mut iter = parts.iter();
        if let Some(opcode) = iter.next() {
            match opcode.as_str() {
                "movb" | "movl" => opcode_with_2_args!(iter, Self::Mov),
                "movzbl" => opcode_with_2_args!(iter, Self::MovZ),
                "xorl" => opcode_with_2_args!(iter, Self::Xor),
                "addb" => opcode_with_2_args!(iter, Self::Adc),
                "incb" => opcode_with_1_arg!(iter, Self::Inc),
                "decb" => opcode_with_1_arg!(iter, Self::Dec),
                "jmp" => opcode_with_1_arg!(iter, Self::Jmp),
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
                (Arg::Literal(l), Arg::VirtualRegister(r)) if l < &0i32  => {
                    writeln!(f,
                        "\tPHA\n\
                         \tSBW VREG_{reg} #{literal}\n\
                         \tPLA"
                         , reg=r, literal=-l)
                },
                _ => writeln!(f, "Unable to generate code for opcode 'ADC' with combination of arguments: '{:?}' and '{:?}'", l, r),
            },
            Self::MovZ(l, r) => match (l, r) {
                (Arg::VirtualRegister(l), Arg::VirtualRegister(r)) if l == r => {
                    // Do nothing
                    Ok(())
                },
                (Arg::Accumulator, Arg::VirtualRegister(r)) => {
                    writeln!(f,
                        "\tSTA VREG_{reg}\n\
                         \tPHA\n\
                         \tLDA #0\n\
                         \tSTA VREG_{reg}+1\n\
                         \tPLA"
                         , reg=r)
                },
                _ => writeln!(f, "Unable to generate code for opcode 'MOVZ' with combination of arguments: '{:?}' and '{:?}'", l, r),
            },
            Self::Mov(l, r) => match (l, r) {
                (Arg::Literal(l), Arg::SumAddress(x, y)) => {
                    writeln!(f,
                        "\tPHA\n\
                         \tTYA\n\
                         \tPHA\n\
                         \tMWA VREG_{op1} TMPW\n\
                         \tADW TMPW VREG_{op2}\n\
                         \tLDY #0\n\
                         \tLDA #{literal}\n\
                         \tSTA (TMPW),y\n\
                         \tPLA\n\
                         \tTAY\n\
                         \tPLA"
                         , literal=l, op1=x, op2=y)
                },
                (Arg::Literal(l), Arg::AbsoluteAddress(a)) => {
                    writeln!(f,
                        "\tPHA\n\
                         \tLDA #{literal}\n\
                         \tSTA {addr}\n\
                         \tPLA"
                         , literal=l, addr=a)
                },
                (Arg::Literal(l), Arg::Accumulator) => {
                    writeln!(f,
                        "\tLDA #{literal}"
                         , literal=l)
                },
                (Arg::Accumulator, Arg::AbsoluteAddress(a)) => {
                    writeln!(f, "\tSTA {}", a)
                },
                (Arg::AbsoluteAddress(a), Arg::Accumulator) => {
                    writeln!(f, "\tLDA {}", a)
                },
                (Arg::Literal(l), Arg::VirtualRegister(r)) => {
                    writeln!(f,
                        "\tPHA\n\
                         \tMWA #{literal} VREG_{reg}\n\
                         \tPLA"
                         , literal=l, reg=r)
                },
                (Arg::Accumulator, Arg::VirtualRegister(r)) => {
                    writeln!(f,
                        "\tPHA\n\
                         \tSTA VREG_{reg}\n\
                         \tLDA #0\n\
                         \tSTA VREG_{reg}+1\n\
                         \tPLA"
                         , reg=r)
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
            Self::Dec(a) => {
                match a {
                    Arg::Accumulator =>{
                        writeln!(f, "\tSEC\n\tSBC #1")
                    }
                    Arg::VirtualRegister(r) =>{
                        writeln!(f, "\tDEW VREG_{reg}", reg=r)
                    }
                    _ => writeln!(f, "Unable to generate code for opcode 'DEC' with argument: '{:?}'", a),
                }
            }
            _ => writeln!(f, "Unable to generate 6502 code for line: {:?}", self),
        }
    }
}

#[derive(Debug)]
struct Transpiler {
    pub vregs: HashSet<char>,
}

impl Transpiler {
    fn add_vreg(&mut self, r: char) {
        self.vregs.insert(r);
    }

    fn insert_if_is_virtual_register(&mut self, arg: &Arg) {
        if let Arg::VirtualRegister(r) = arg {
            self.add_vreg(*r)
        }
    }

    fn check_for_virtual_registers(&mut self, asm_line: &AsmLine) {
        match &asm_line {
            AsmLine::Xor(arg1, arg2)
            | AsmLine::Adc(arg1, arg2)
            | AsmLine::Mov(arg1, arg2)
            | AsmLine::MovZ(arg1, arg2) => {
                self.insert_if_is_virtual_register(arg1);
                self.insert_if_is_virtual_register(arg2);
            }
            AsmLine::Inc(arg) | AsmLine::Dec(arg) | AsmLine::Jmp(arg) => {
                self.insert_if_is_virtual_register(arg);
            }
            _ => {}
        };
    }
}

fn main() -> Result<(), std::io::Error> {
    let mut transpiler = Transpiler {
        vregs: HashSet::new(),
    };

    let file = File::open(FILENAME)?;
    let file = BufReader::new(&file);

    eprintln!("Parsing input file...");
    let input: Vec<AsmLine> = file
        .lines()
        .skip(1)
        .enumerate()
        .map(|(num, l)| {
            print!("Line {:4}:\t", num);
            l.expect("Parse error")
        })
        .map(|s| {
            println!("{}", s);
            let s = s.parse::<AsmLine>().expect("Parse error");
            transpiler.check_for_virtual_registers(&s);
            s
        })
        .collect();

    eprintln!("Parsing complete.");
    eprintln!();

    eprintln!("Generating 6502 code...");
    transpiler
        .vregs
        .iter()
        .for_each(|reg| println!(".ZPVAR .WORD VREG_{}", reg));
    println!("\t.ZPVAR .WORD TMPW");
    println!("\tORG $2000");
    input.into_iter().for_each(|l| print!("{}", l));
    eprintln!("Code generation complete.");
    eprintln!();

    Ok(())
}
