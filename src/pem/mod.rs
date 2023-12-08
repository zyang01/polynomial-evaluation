//! Polynomial Evaluation Machine (PEM)

mod instruction;

pub(crate) use instruction::Instruction;

mod types {
    /// Register ID in range 0..=7
    pub type Reg = u32;

    /// Memory address in range 0..2^32
    pub type Addr = u32;

    /// 32-bit numeric constant
    pub type Const = u32;
}
