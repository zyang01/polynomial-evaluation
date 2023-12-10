//! Polynomial Evaluation Machine (PEM)

mod inflight_operation;
mod instruction;
mod machine;

use std::rc::Rc;

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
}

#[derive(Debug, Clone)]
pub(crate) enum Expr {
    Const(u32),
    Value(String),
    Add(Rc<Expr>, Rc<Expr>),
    Sub(Rc<Expr>, Rc<Expr>),
    Mul(Rc<Expr>, Rc<Expr>),
}

pub(crate) type RcExpr = Rc<Expr>;

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.eval())
    }
}

impl Expr {
    pub fn new_const(value: u32) -> RcExpr {
        Rc::new(Self::Const(value))
    }

    pub fn new_value(value: &str) -> RcExpr {
        Rc::new(Self::Value(value.to_string()))
    }

    pub fn add(src1: RcExpr, src2: RcExpr) -> RcExpr {
        Rc::new(Self::Add(src1, src2))
    }

    pub fn sub(src1: RcExpr, src2: RcExpr) -> RcExpr {
        Rc::new(Self::Sub(src1, src2))
    }

    pub fn mul(src1: RcExpr, src2: RcExpr) -> RcExpr {
        Rc::new(Self::Mul(src1, src2))
    }

    pub fn eval(&self) -> String {
        match self {
            Self::Const(constant) => constant.to_string(),
            Self::Value(val) => val.clone(),
            Self::Add(src1, src2) => format!("({} + {})", src2.eval(), src1.eval()),
            Self::Sub(src1, src2) => format!("({} - {})", src1.eval(), src2.eval()),
            Self::Mul(src1, src2) => format!("({} * {})", src1.eval(), src2.eval()),
        }
    }
}
