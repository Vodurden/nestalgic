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

- nestest test rom: https://www.qmtpro.com/~nes/misc/
