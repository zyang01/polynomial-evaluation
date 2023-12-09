//! Polynomial Evaluation Machine (PEM)

mod inflight_operation;
mod instruction;
mod machine;

pub(crate) use instruction::Instruction;
pub(crate) use machine::Machine;

/// PEM primitive types
pub(crate) mod types {
    /// Register ID in range 0..=7
    #[derive(Debug, Clone, Copy)]
    pub struct Reg(pub u32);

    /// Memory address in range 0..2^32
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
    pub struct Addr(pub u32);

    /// 32-bit numeric constant
    #[derive(Debug, Clone, Copy)]
    pub struct Const(pub u32);

    #[derive(Debug, Clone)]
    pub struct Value(pub String);
}
