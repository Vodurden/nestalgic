/// Status represents the processor status flag, `p` on the `MOS6502`
///
/// Each bit in `p` has a different meaning:
///
/// ```text
/// +---+---+---+---+---+---+---+---+
/// | N | V |   | B | D | I | Z | C |
/// +---+---+---+---+---+---+---+---+
///   |   |   |   |   |   |   |   |
///   |   |   |   |   |   |   |   \-------- CARRY
///   |   |   |   |   |   |   |
///   |   |   |   |   |   |   \------------ ZERO RESULT
///   |   |   |   |   |   |
///   |   |   |   |   |   \---------------- INTERRUPT DISABLE
///   |   |   |   |   |
///   |   |   |   |   \-------------------- DECIMAL MODE
///   |   |   |   |
///   |   |   |   \------------------------ BREAK COMMAND
///   |   |   |
///   |   |   \---------------------------- EXPANSION
///   |   |
///   |   \-------------------------------- OVERFLOW
///   |
///   \------------------------------------ NEGATIVE RESULT
/// ```
///
/// Flag descriptions:
///
/// - `C` is the carry flag which is modified by specific arithmetic operations. It's used as the "ninth bit" for many arithmetic operations.
/// - `Z` is automatically set during any movement or calculation hen the 8 bits of the resulting operation are 0.
/// - `I` is the interrupt disable flag. When set it disables the effect of the interrupt request pin.
/// - `D` is the decimal mode flag. This flag is ignored on the NES. On the 6502 is makes add/subtract operations work on the decimal representation of numbers
/// - `B` is only set by the processor and is used to determine if an interrupt was caused by the `BRK` command or a real interrupt. It's always 0 in `P` but exists when `P` is pushed to the stack using `PHP`
/// - ` ` is the expansion bit. It's unused and always set to 1.
/// - `V` is set when addition/subtraction overflows.
/// - `N` is set after all data movements or arithmetic. If the resultant value is negative this bit will be set to `1`.
///
/// Gotchas:
///
/// - `B` doesn't exist in `P`. It is _only_ set when `P` is pushed to the stack from `BRK` or `PHP`.
/// - `B` is ignored when reading from the stack into `P`
/// - ` ` (unused) is _always_ set to 1.

#[derive(Eq, PartialEq, Debug)]
pub struct Status(pub u8);

impl Status {
    pub fn get(&self, flag: StatusFlag) -> bool {
        let bit = flag as u8;

        (self.0 & (1 << bit)) != 0
    }

    pub fn set(&mut self, flag: StatusFlag, value: bool) {
        let bit = flag as u8;
        if value {
            self.0 |= 1 << bit;
        } else {
            self.0 &= !(1 << bit);
        }
    }

    pub fn with(&mut self, flag: StatusFlag, value: bool) -> &mut Self {
        self.set(flag, value);
        self
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum StatusFlag {
    Carry = 0,
    Zero = 1,
    InterruptDisable = 2,
    DecimalMode = 3,
    Break = 4,
    Unused = 5,
    Overflow = 6,
    Negative = 7,
}

impl StatusFlag {
    pub fn variants() -> impl Iterator<Item = StatusFlag> {
        [
            StatusFlag::Carry,
            StatusFlag::Zero,
            StatusFlag::InterruptDisable,
            StatusFlag::DecimalMode,
            StatusFlag::Break,
            StatusFlag::Unused,
            StatusFlag::Overflow,
            StatusFlag::Negative,
        ].iter().copied()
    }
}
