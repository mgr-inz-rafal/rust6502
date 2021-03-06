use std::{fmt, str::FromStr};

use crate::asm_line::AsmLineError;

#[derive(Debug)]
pub(in crate) enum Arg {
    Literal(i32),
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
            "ebx" | "bl"=> Ok('B'),
            "edx" | "dl" => Ok('D'),
            "esi" => Ok('S'),
            _ => Err(AsmLineError::MalformedRegisterName(name.to_string())),
        }
    }
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Label(s) => write!(f, "{}", s),
            _ => write!(f, "Unable to generate 6502 code for argument: {:?}", self),
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
                .and_then(|c| Ok(Self::VirtualRegister(c))),
                '.' => Ok(Self::Label({
                    it.skip(1).filter(|c| *c != ',').collect::<String>()
                })),
                '(' => {
                    let args: String = it.collect();
                    let args = args.trim_end_matches(')');
                    let args = args.trim_start_matches('(');
                    let args: Vec<String> = args.split(',').map(ToString::to_string).collect();

                    // TODO: Simplification: edx => D, ecx => C, etc.
                    Ok(Self::SumAddress(
                        args[0].chars().nth(2).unwrap(),
                        args[1].chars().nth(2).unwrap(),
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
            Err(AsmLineError::UnknownError)
        }
    }
}
