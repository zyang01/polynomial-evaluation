//! Polynomial Evaluation Machine (PEM)

mod inflight_operation;
mod instruction;
mod machine;

use std::{
    cell::RefCell,
    ops::{Add, Mul, Sub},
    rc::Rc,
};

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
enum Expr {
    Const(u32),
    Value(String),
    Add(RcRefCellExpr, RcRefCellExpr),
    Sub(RcRefCellExpr, RcRefCellExpr),
    Mul(RcRefCellExpr, RcRefCellExpr),
}

type RcRefCellExpr = Rc<RefCell<Expr>>;

#[derive(Debug, Clone)]
pub(crate) struct ExprWrapper(RcRefCellExpr);

impl std::fmt::Display for ExprWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.eval())
    }
}

impl From<String> for ExprWrapper {
    fn from(value: String) -> Self {
        Self::new(Expr::Value(value))
    }
}

impl From<&str> for ExprWrapper {
    fn from(value: &str) -> Self {
        Self::new(Expr::Value(value.to_string()))
    }
}

impl From<u32> for ExprWrapper {
    fn from(value: u32) -> Self {
        Self::new(Expr::Const(value))
    }
}

impl Add for &ExprWrapper {
    type Output = ExprWrapper;

    fn add(self, rhs: &ExprWrapper) -> Self::Output {
        ExprWrapper::new(Expr::Add(self.0.clone(), rhs.0.clone()))
    }
}

impl Sub for &ExprWrapper {
    type Output = ExprWrapper;

    fn sub(self, rhs: &ExprWrapper) -> Self::Output {
        ExprWrapper::new(Expr::Sub(self.0.clone(), rhs.0.clone()))
    }
}

impl Mul for &ExprWrapper {
    type Output = ExprWrapper;

    fn mul(self, rhs: &ExprWrapper) -> Self::Output {
        ExprWrapper::new(Expr::Mul(self.0.clone(), rhs.0.clone()))
    }
}

impl ExprWrapper {
    fn new(expr: Expr) -> Self {
        Self(Rc::new(RefCell::new(expr)))
    }

    fn eval_expr(expr: &RcRefCellExpr) -> String {
        let value = match *expr.borrow() {
            Expr::Const(constant) => constant.to_string(),
            Expr::Value(ref val) => val.clone(),
            Expr::Add(ref src1, ref src2) => {
                format!("({} + {})", Self::eval_expr(src2), Self::eval_expr(src1))
            }
            Expr::Sub(ref src1, ref src2) => {
                format!("({} - {})", Self::eval_expr(src1), Self::eval_expr(src2))
            }
            Expr::Mul(ref src1, ref src2) => {
                format!("({} * {})", Self::eval_expr(src1), Self::eval_expr(src2))
            }
        };
        *expr.borrow_mut() = Expr::Value(value.clone());
        value
    }

    pub fn eval(&self) -> String {
        Self::eval_expr(&self.0)
    }
}
