# Introduction

Nestalgic is a Nintendo Entertainment System (NES) emulator written in Rust.

# Getting Started

Nestalgic uses `nix` to sandbox the project dependencies. To start the development environment run `nix-shell`.

# Releasing

Nestalgic supports Linux and Windows:

- To build Linux binaries run `nix build -f release.nix nestalgic-linux`.
- To build Windows binaries run `nix-build -f release.nix nestalgic-windows`

The build outputs to `./result/bin/`.

# References

## CPU (6502)

- Instruction Reference: http://www.obelisk.me.uk/6502/reference.html#CMP
- Address Mode Timing Reference: http://atarihq.com/danb/files/64doc.txt
- How the overflow flag works: http://www.6502.org/tutorials/vflag.html
- nestest test rom: https://www.qmtpro.com/~nes/misc/
