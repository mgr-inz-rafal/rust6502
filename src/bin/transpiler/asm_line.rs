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
    MovZ(Arg, Arg),
    Cmp(Arg, Arg),
    CMov(Arg, Arg),
    Inc(Arg),
    Dec(Arg),
    Jmp(Arg),
    Push(Arg),
}

impl AsmLine {
    fn args<'a, I>(parts: I, expected_count: usize) -> Result<Vec<String>, AsmLineError>
    where
        I: Iterator<Item = &'a String>,
    {
        let mut result: Vec<String> = vec![];

        let args: Vec<&'a String> = parts.collect();
        match args.len() {
            1 => result.push(args.get(0)?.to_string()),
            2 => {
                for i in 0..=1 {
                    result.push(args.get(i)?.to_string())
                }
            },
            3 => {
                // Special case for lines like `movb $10, (%eax, %edx)`
                // where `(%eax, %edx)` must be combined into single argument
                result.push(args.get(0)?.to_string());
                let second = args.get(1)?.to_string();
                if second.starts_with('(') {
                    result.push(format!("{}{}", second, args.get(2)?));
                }
            }
            _ => panic!("Argument parsing error")
        }

        if result.len() == expected_count {
            Ok(result)
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
                "cmpb" => opcode_with_2_args!(iter, Self::Cmp),
                "cmovel" => opcode_with_2_args!(iter, Self::CMov),
                "incb" => opcode_with_1_arg!(iter, Self::Inc),
                "decb" => opcode_with_1_arg!(iter, Self::Dec),
                "pushl" => opcode_with_1_arg!(iter, Self::Push),
                "jmp" => opcode_with_1_arg!(iter, Self::Jmp),
                _ => return Err(AsmLineError::UnknownOpcode(opcode.to_string())),
            }
        }

        Err(AsmLineError::UnknownError)
    }
}

impl fmt::Display for AsmLine {
    #[allow(clippy::many_single_char_names)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Label(l) => writeln!(f, "{}", l),
            Self::Push(Arg::VirtualRegister(r)) => 
            writeln!(f,
                "\tSTA TMPW\n\
                \tLDA VREG_{reg}\n\
                \tPHA\n\
                \tLDA VREG_{reg}+1\n\
                \tPHA\n\
                \tLDA TMPW"
                , reg=r),
            Self::Jmp(l) => writeln!(f, "\tJMP {}", l),
//            Self::Jmp(l) => writeln!(f, "\tJSR SYNCHRO\n\tJMP {}", l),
            Self::Xor(Arg::VirtualRegister(l), Arg::VirtualRegister(r)) if l == r => 
                writeln!(f,
                    "\tPHA\n\
                     \tLDA #0\n\
                     \tSTA VREG_{reg}\n\
                     \tSTA VREG_{reg}+1\n\
                     \tPLA"
                    , reg=l),
            Self::Cmp(Arg::Literal(l), Arg::VirtualRegister(r)) => 
                // TODO: Cheat - we do the 8-bit comparison only
                writeln!(f,
                    "\tPHA\n\
                     \tLDA VREG_{reg}\n\
                     \tCMP #{lit}\n\
                     \tjsr LAST_CMP_EQUAL\n\
                     \tPLA"
                    , reg=r, lit=l),
            Self::Adc(l, r) => match (l, r) {
                (Arg::Literal(l), Arg::VirtualRegister(r)) if l < &0i32 => {
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
                (Arg::AbsoluteAddress(a), Arg::VirtualRegister(r)) => {
                    writeln!(f,
                        "\tPHA\n\
                         \tCLC\n\
                         \tLDA VREG_{reg}\n\
                         \tADC {addr}\n\
                         \tSTA VREG_{reg}\n\
                         \tLDA VREG_{reg}+1\n\
                         \tADC #0\n\
                         \tSTA VREG_{reg}+1\n\
                         \tPLA"
                         , addr=a, reg=r)
                },
                _ => writeln!(f, "Unable to generate code for opcode 'ADC' with combination of arguments: '{:?}' and '{:?}'", l, r),
            },
            Self::MovZ(l, r) => match (l, r) {
                (Arg::VirtualRegister(l), Arg::VirtualRegister(r)) if l == r => {
                    // Do nothing
                    Ok(())
                },
                (Arg::VirtualRegister(l), Arg::VirtualRegister(r)) => {
                    writeln!(f,
                        "\tPHA\n\
                         \tLDA VREG_{source}\n\
                         \tSTA VREG_{target}\n\
                         \tLDA VREG_{source}+1\n\
                         \tSTA VREG_{target}+1\n\
                         \tPLA"
                         , source=l, target=r)
                },
                _ => writeln!(f, "Unable to generate code for opcode 'MOVZ' with combination of arguments: '{:?}' and '{:?}'", l, r),
            },
            Self::CMov(l, r) => match (l, r) {
                (Arg::VirtualRegister(l), Arg::VirtualRegister(r)) => {
                    writeln!(f,
                        "\tPHA\n\
                         \tLDA LAST_CMP\n\
                         \tCMP #1\n\
                         \tBEQ @+\n\
                         \tLDA VREG_{source}\n\
                         \tSTA VREG_{target}\n\
                         \tLDA VREG_{source}+1\n\
                         \tSTA VREG_{target}+1\n\
                         @\tPLA"
                         , source=l, target=r)
                },
                _ => writeln!(f, "Unable to generate code for opcode 'CMov' with combination of arguments: '{:?}' and '{:?}'", l, r),
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
                (Arg::VirtualRegister(l), Arg::VirtualRegister(r)) => {
                    writeln!(f,
                        "\tPHA\n\
                         \tLDA VREG_{source}\n\
                         \tSTA VREG_{target}\n\
                         \tLDA VREG_{source}+1\n\
                         \tSTA VREG_{target}+1\n\
                         \tPLA"
                         , source=l, target=r)
                },
                _ => writeln!(f, "Unable to generate code for opcode 'MOV' with combination of arguments: '{:?}' and '{:?}'", l, r),
            },
            Self::Inc(a) => {
                match a {
                    Arg::VirtualRegister(r) => {
                        writeln!(f,
                            "\tPHA\n\
                             \tCLC\n\
                             \tLDA VREG_{reg}\n\
                             \tADC #<1\n\
                             \tSTA VREG_{reg}\n\
                             \tLDA VREG_{reg}+1\n\
                             \tADC #>1\n\
                             \tSTA VREG_{reg}+1\n\
                             \tPLA"
                             , reg=r)
                        }
                    _ => writeln!(f, "Unable to generate code for opcode 'INC' with argument: '{:?}'", a),
                }
            }
            Self::Dec(a) => {
                match a {
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
