//! Polynomial Evaluation Machine (PEM)

mod inflight_operation;
mod instruction;
mod machine;

pub(crate) use instruction::Instruction;
pub(crate) use machine::Machine;

/// PEM primitive types
pub(crate) mod types {
    /// Register ID in range 0..=7
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Reg(pub u32);

    impl std::fmt::Display for Reg {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Reg({})", self.0)
        }
    }

    /// Memory address in range 0..2^32
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
    pub struct Addr(pub u32);

    impl std::fmt::Display for Addr {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Addr({})", self.0)
        }
    }

    /// 32-bit numeric constant
    #[derive(Debug, Clone, Copy)]
    pub struct Const(pub u32);

    impl std::fmt::Display for Const {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Const({})", self.0)
        }
    }

    #[derive(Debug, Clone)]
    pub struct Value(pub String);
}
