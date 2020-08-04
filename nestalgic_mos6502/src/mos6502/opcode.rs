use std::fmt;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Opcode {
    // =====================================================================================
    // ================================ Register Operations ================================
    // =====================================================================================

    /// Load a byte of memory into `A`
    LDA,

    /// Load a byte of memory into `X`
    LDX,

    /// Load a byte of memory into `Y`
    LDY,

    /// Load a byte of memory into `A` and `X`
    ///
    /// This is an "Unofficial" opcode but shows up in some binaries regardless
    LAX,

    /// Store the contents of `A` into memory
    STA,

    /// Store the contents of `X` into memory
    STX,

    /// Store the contents of `Y` into memory
    STY,

    /// Load `A & X` into a byte of memory
    ///
    /// This is an "Unofficial" opcode but shows up in some binaries regardless
    SAX,

    /// Copy the contents of `A` into `X`
    TAX,

    /// Copy the contents of `A` into `Y`
    TAY,

    /// Copy the contents of `X` into `A`
    TXA,

    /// Copy the contents of `Y` into `A`
    TYA,

    // =====================================================================================
    // =================================== Stack Operations ================================
    // =====================================================================================

    /// Copy `SP` into `X`
    TSX,

    /// Copy `X` into `SP`
    TXS,

    /// Push `A` onto the stack
    PHA,

    /// Push `P` onto the stack.
    PHP,

    /// Pull the current stack value into `A`
    PLA,

    /// Pull the current stack value into `P`
    PLP,

    // =====================================================================================
    // ================================= Logical Operations ================================
    // =====================================================================================

    /// Logical AND. Set `A` to `A & M` where `M` is the memory targeted by this instruction
    AND,

    /// Exclusive OR. Set `A` to `A XOR M` where `M` is the memory targeted by this instruction
    EOR,

    /// Logical OR. Set `A` to `A | M` where `M` is the memory targeted by this instruction
    ORA,

    /// Bit Test. Test if one or more bits are set in the target memory location.
    ///
    /// `A` is used as a mask which is AND'ed with the target memory location. The results
    /// are written into `P` under the `Zero`, `Overflow` and `Negative` flags.
    BIT,

    // =====================================================================================
    // ====================================== Arithmetic ===================================
    // =====================================================================================

    /// Add with Carry. Add the target memory location to the accumulator.
    ADC,

    /// Subtract with Carry. Subtract the target memory location from the accumulator.
    SBC,

    /// Compare the accumulator with the target memory location.
    ///
    /// Sets `Zero` in `P` if both values are equal
    /// Sets `Carry` in `P` if `Accumulator >= Value`
    /// Sets `Negative` in `P` if `Accumulator < Value`
    CMP,

    /// Same as `CMP` but compares `X` and the target memory location
    CPX,

    /// Same as `CMP` but compares `Y` and the target memory location
    CPY,

    // =====================================================================================
    // =============================== Increments & Decrements =============================
    // =====================================================================================

    /// Increment Memory. Add 1 to the target memory location
    INC,

    /// Increment `X`. Add 1 to `X`
    INX,

    /// Increment `Y`. Add 1 to `Y`
    INY,

    /// Increment Memory then subtract the result from `A`
    ///
    /// This is an unofficial opcode
    ISC,

    /// Decrement Memory. Subtract 1 to the target memory location
    DEC,

    /// Decrement `X`. Subtract 1 from `X`
    DEX,

    /// Decrement `Y`. Subtract 1 from `Y`
    DEY,

    /// Decrement Memory the compare the results with `A`
    ///
    /// This is an unofficial opcode
    DCP,

    // =====================================================================================
    // ======================================= Shifts ======================================
    // =====================================================================================

    /// Arithmetic Shift Left: Shift the targeted memory one bit to the left
    ///
    /// Bit 7 is placed in `Carry`
    ASL,

    /// Logical Shift Right: Shift the targeted memory one bit to the right
    ///
    /// Bit 0 is placed in `Carry`
    LSR,

    /// Rotate Left. Shift the bits of the targeted memory one place to the left.
    ///
    /// Bit 0 is filled with the current value of `Carry` and bit 7 becomes the new `Carry`
    ROL,

    /// Rotate Right. Shift the bits of the targeted memory one place to the right.
    ///
    /// Bit 7 is filled with the current value of `Carry` and bit 0 becomes the new `Carry`
    ROR,

    /// Shift the targeted memory one bit to the left then OR the result with `A`
    ///
    /// Also known as `ASO`
    ///
    /// This is an unofficial opcode
    SLO,

    /// Shift the targeted memory one bit to the left then XOR the result with `A`
    ///
    /// Also known as `LSE`
    ///
    /// This is an unoffiical opcode
    ///
    /// TODO: Finish instruction table for this opcode
    SRE,

    /// Rotate the targeted memory one bit to the right then AND the result with `A`
    ///
    /// This is an unofficial opcode
    RLA,

    // =====================================================================================
    // =================================== Jumps & Calls ===================================
    // =====================================================================================

    JMP,
    JSR,
    RTS,


    // =====================================================================================
    // ====================================== Branches =====================================
    // =====================================================================================

    /// Branch Carry Set: Set `PC` to `address` if `Carry` flag is `true`.
    BCS,

    /// Branch Carry Clear: Set `PC` to `address` if the `Carry` flag is `false`.
    BCC,

    /// Branch Equal: Set `PC` to `address` if equal (i.e. the `Zero` flag is `true`)
    BEQ,

    /// Branch Not Equal: Set `PC` to `address` if not equal (i.e. the `Zero` flag is `false`)
    BNE,

    /// Branch If Minus: Set `PC` to `address` if minus (i.e. the `Negative` flag is `true`)
    BMI,

    /// Branch If Positive: Set `PC` to `address` if positive (i.e. the `Negative` flag is `false`)
    BPL,

    /// Branch If Overflow Clear: Set `PC` to `address` if `Overflow` is `false`.
    BVC,

    /// Branch If Overflow Set: Set `PC` to `address` if `Overflow` is `true`
    BVS,


    // =====================================================================================
    // ================================ Status Flag Changes ================================
    // =====================================================================================

    /// Clear Carry Flag: Set `Carry` in `P` to `false`
    CLC,

    /// Clear Decimal Mode: Set `DecimalMode` in `P` to `false`
    CLD,

    /// Clear Interrupt Disable: Set `InterruptDisable` in `P` to `false`
    CLI,

    /// Clear Overflow Flag: Set `Overflow` in `P` to `false`
    CLV,

    /// Set Carry Flag: Set `Carry` in `P` to `true`
    SEC,

    /// Set Decimal Mode: Set `DecimalMode` in `P` to `true`
    SED,

    /// Set Interrupt Disable: Set `InterruptDisable` in `P` to `true`
    SEI,

    // =====================================================================================
    // ================================== System Functions =================================
    // =====================================================================================
    BRK,

    /// No Operation: Do nothing, skip to next instruction
    NOP,

    /// Return from Interrupt: Pull `P` from the stack followed by `PC`
    RTI,
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
