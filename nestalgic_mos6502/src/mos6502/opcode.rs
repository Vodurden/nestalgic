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

    /// Store the contents of `A` into memory
    STA,

    /// Store the contents of `X` into memory
    STX,

    /// Store the contents of `Y` into memory
    STY,

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
    AND,
    EOR,
    ORA,
    BIT,

    // =====================================================================================
    // ====================================== Arithmetic ===================================
    // =====================================================================================
    ADC,
    SBC,
    CMP,
    CPX,
    CPY,

    // =====================================================================================
    // =============================== Increments & Decrements =============================
    // =====================================================================================
    INC,
    INX,
    INY,
    DEC,
    DEX,
    DEY,

    // =====================================================================================
    // ======================================= Shifts ======================================
    // =====================================================================================

    ASL,
    LSR,
    ROL,
    ROR,

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

    RTI,
}
