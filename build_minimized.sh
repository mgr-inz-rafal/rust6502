#!/bin/bash
RUSTFLAGS="-C opt-level=s" cargo asm main::asm6502 --no-color --asm-style att --target i686-unknown-linux-gnu >output.asm
