#!/bin/bash
RUSTFLAGS="-C opt-level=s" cargo asm transpiler::source::asm6502_source --no-color --asm-style att --target i686-unknown-linux-gnu >output_x86.asm
