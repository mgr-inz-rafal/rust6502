#![feature(llvm_asm, const_if_match, try_trait)]
mod arg;
mod asm_line;
mod source;

use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

use arg::Arg;
use asm_line::AsmLine;

const FILENAME: &str = "output.asm";

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
    if env::args().find(|arg| arg == "--nocrash").is_none() {
        let _ = source::asm6502_source();
    }

    let mut transpiler = Transpiler {
        vregs: HashSet::new(),
    };

    let file = File::open(FILENAME)?;
    let file = BufReader::new(&file);

    eprintln!("Parsing input file...");
    println!("\tORG $2000");
    file.lines()
        .skip(1)
        .enumerate()
        .map(|(num, l)| {
            print!("; Line {:4}:\t", num);
            l.expect("Parse error")
        })
        .map(|s| {
            println!("{}", s);
            let s = s.parse::<AsmLine>().expect("Parse error");
            transpiler.check_for_virtual_registers(&s);
            s
        })
        .for_each(|l| print!("{}\n", l));

    const ZERO_PAGE_BASE: usize = 0x80;
    const VIRTUAL_REGISTERS_BASE: usize = ZERO_PAGE_BASE + 3;
    println!("TMPW equ {}", ZERO_PAGE_BASE);
    println!("LAST_CMP equ {}", ZERO_PAGE_BASE+2);
    transpiler
        .vregs
        .iter()
        .enumerate()
        .for_each(|(index, reg)| {
            println!("VREG_{} equ {}", reg, VIRTUAL_REGISTERS_BASE + (index << 1));
        });

    // Add runtime :)
    println!(r#"
PAL     = $D014
VCOUNT  = $D40B
SYNCHRO
            lda PAL
            beq SYN_0
            lda #120	; NTSC
            jmp SYN_1
SYN_0       lda #145	; PAL
SYN_1       cmp VCOUNT
            bne SYN_1
            rts        

LAST_CMP_EQUAL
        BEQ @+
        LDA #1
        STA LAST_CMP
        RTS
@       LDA #0
        STA LAST_CMP
        RTS
    "#);

    Ok(())
}
