use std::{fmt, option::NoneError, str::FromStr};

use crate::arg::Arg;

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
pub(in crate) enum AsmLineError {
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

#[derive(Debug)]
pub(in crate) enum AsmLine {
    Label(String),
    Xor(Arg, Arg),
    Adc(Arg, Arg),
    Mov(Arg, Arg),
    CMov(Arg, Arg),
    MovZ(Arg, Arg),
    Cmp(Arg, Arg),
    Push(Arg),
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
                "cmovel" => opcode_with_2_args!(iter, Self::CMov),
                "movzbl" => opcode_with_2_args!(iter, Self::MovZ),
                "xorl" => opcode_with_2_args!(iter, Self::Xor),
                "addb" => opcode_with_2_args!(iter, Self::Adc),
                "cmpb" => opcode_with_2_args!(iter, Self::Cmp),
                "incb" => opcode_with_1_arg!(iter, Self::Inc),
                "decb" => opcode_with_1_arg!(iter, Self::Dec),
                "jmp" => opcode_with_1_arg!(iter, Self::Jmp),
                "pushl" => opcode_with_1_arg!(iter, Self::Push),
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
                         \tSEC\n\
                         \tLDA VREG_{reg}\n\
                         \tSBC #<{literal}\n\
                         \tSTA VREG_{reg}\n\
                         \tLDA VREG_{reg}+1\n\
                         \tSBC #>{literal}\n\
                         \tSTA VREG_{reg}+1\n\
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
                         \tLDA VREG_{op1}\n\
                         \tSTA TMPW\n\
                         \tLDA VREG_{op1}+1\n\
                         \tSTA TMPW+1\n\
                         \tCLC\n\
                         \tLDA TMPW\n\
                         \tADC VREG_{op2}\n\
                         \tSTA TMPW\n\
                         \tLDA TMPW+1\n\
                         \tADC VREG_{op2}+1\n\
                         \tSTA TMPW+1\n\
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
                         \tLDA #<{literal}\n\
                         \tSTA VREG_{reg}\n\
                         \tLDA #>{literal}\n\
                         \tSTA VREG_{reg}+1\n\
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
