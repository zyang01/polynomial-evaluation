use std::{
    ops::{Add, Mul, Sub},
    rc::Rc,
};

#[derive(Debug, Clone)]
enum EvaluatedExprKind {
    Numeric(u32),
    Value(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    Add,
    Sub,
    Mul,
    NumericOrSymbolicVariable,
}

#[derive(Debug, Clone)]
enum Expr {
    Const(u32),
    SymbolicVariable(String),
    Add(RcExpr, RcExpr),
    Sub(RcExpr, RcExpr),
    Mul(RcExpr, RcExpr),
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Const(constant) => write!(f, "{}", constant),
            Expr::SymbolicVariable(value) => write!(f, "{}", value),
            Expr::Add(lhs, rhs) => write!(f, "({} + {})", lhs, rhs),
            Expr::Sub(lhs, rhs) => write!(f, "({} - {})", lhs, rhs),
            Expr::Mul(lhs, rhs) => write!(f, "({} * {})", lhs, rhs),
        }
    }
}

type RcExpr = Rc<Expr>;

#[derive(Debug)]
struct EvaluatedExpr {
    kind: EvaluatedExprKind,
    precedence: Precedence,
}

impl std::fmt::Display for EvaluatedExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            EvaluatedExprKind::Numeric(constant) => write!(f, "{}", constant),
            EvaluatedExprKind::Value(ref value) => write!(f, "{}", value),
        }
    }
}

impl Add for EvaluatedExpr {
    type Output = EvaluatedExpr;

    fn add(self, rhs: Self) -> Self::Output {
        match (&self.kind, &rhs.kind, &rhs.precedence) {
            (EvaluatedExprKind::Numeric(lhs), EvaluatedExprKind::Numeric(rhs), _) => Self {
                kind: EvaluatedExprKind::Numeric(lhs.wrapping_add(*rhs)),
                precedence: Precedence::NumericOrSymbolicVariable,
            },
            _ => Self {
                kind: EvaluatedExprKind::Value(format!("{} + {}", self, rhs)),
                precedence: Precedence::Add,
            },
        }
    }
}

impl Sub for EvaluatedExpr {
    type Output = EvaluatedExpr;

    fn sub(self, rhs: Self) -> Self::Output {
        match (&self.kind, &rhs.kind, &rhs.precedence) {
            (EvaluatedExprKind::Numeric(lhs), EvaluatedExprKind::Numeric(rhs), _) => Self {
                kind: EvaluatedExprKind::Numeric(lhs.wrapping_sub(*rhs)),
                precedence: Precedence::NumericOrSymbolicVariable,
            },
            (_, _, Precedence::Add) | (_, _, Precedence::Sub) => Self {
                // ((A + B) - (C - D)) = A + B - (C - D)
                kind: EvaluatedExprKind::Value(format!("{} - ({})", self, rhs)),
                precedence: Precedence::Sub,
            },
            _ => Self {
                kind: EvaluatedExprKind::Value(format!("{} - {}", self, rhs)),
                precedence: Precedence::Sub,
            },
        }
    }
}

impl Mul for EvaluatedExpr {
    type Output = EvaluatedExpr;

    fn mul(self, rhs: Self) -> Self::Output {
        match (&self.kind, &rhs.kind) {
            (EvaluatedExprKind::Numeric(lhs), EvaluatedExprKind::Numeric(rhs)) => Self {
                kind: EvaluatedExprKind::Numeric(lhs.wrapping_mul(*rhs)),
                precedence: Precedence::NumericOrSymbolicVariable,
            },
            _ => {
                let lhs = match self.precedence {
                    Precedence::Add | Precedence::Sub => format!("({})", self),
                    _ => format!("{}", self),
                };
                let rhs = match rhs.precedence {
                    Precedence::Add | Precedence::Sub => format!("({})", rhs),
                    _ => format!("{}", rhs),
                };
                Self {
                    kind: EvaluatedExprKind::Value(format!("{} * {}", lhs, rhs)),
                    precedence: Precedence::Mul,
                }
            }
        }
    }
}

impl From<&RcExpr> for EvaluatedExpr {
    fn from(expr: &RcExpr) -> Self {
        match expr.as_ref() {
            Expr::Const(constant) => Self {
                kind: EvaluatedExprKind::Numeric(*constant),
                precedence: Precedence::NumericOrSymbolicVariable,
            },
            Expr::SymbolicVariable(value) => Self {
                kind: EvaluatedExprKind::Value(value.to_string()),
                precedence: Precedence::NumericOrSymbolicVariable,
            },
            Expr::Add(lhs, rhs) => Self::from(lhs) + rhs.into(),
            Expr::Sub(lhs, rhs) => Self::from(lhs) - rhs.into(),
            Expr::Mul(lhs, rhs) => Self::from(lhs) * rhs.into(),
        }
    }
}

impl From<u32> for EvaluatedExpr {
    fn from(value: u32) -> Self {
        Self {
            kind: EvaluatedExprKind::Numeric(value),
            precedence: Precedence::NumericOrSymbolicVariable,
        }
    }
}

impl From<&str> for EvaluatedExpr {
    fn from(value: &str) -> Self {
        Self {
            kind: EvaluatedExprKind::Value(value.to_string()),
            precedence: Precedence::NumericOrSymbolicVariable,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ExprWrapper(RcExpr);

impl std::fmt::Display for ExprWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.weak_eval())
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
        ExprWrapper::new(Expr::Add(Rc::clone(&self.0), Rc::clone(&rhs.0)))
    }
}

impl Sub for &ExprWrapper {
    type Output = ExprWrapper;

    fn sub(self, rhs: &ExprWrapper) -> Self::Output {
        ExprWrapper::new(Expr::Sub(Rc::clone(&self.0), Rc::clone(&rhs.0)))
    }
}

impl Mul for &ExprWrapper {
    type Output = ExprWrapper;

    fn mul(self, rhs: &ExprWrapper) -> Self::Output {
        ExprWrapper::new(Expr::Mul(Rc::clone(&self.0), Rc::clone(&rhs.0)))
    }
}

impl ExprWrapper {
    fn new(expr: Expr) -> Self {
        Self(Rc::new(expr))
    }

    pub fn from_symbolic_variable<S: Into<String>>(value: S) -> Self {
        Self::new(Expr::SymbolicVariable(value.into()))
    }

    pub fn weak_eval(&self) -> String {
        self.0.to_string()
    }

    pub fn strong_eval(&self) -> String {
        EvaluatedExpr::from(&self.0).to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_strong_eval_wraparound() {
        let EvaluatedExpr {
            kind: EvaluatedExprKind::Numeric(value),
            precedence: Precedence::NumericOrSymbolicVariable,
        } = EvaluatedExpr::from(200u32) + EvaluatedExpr::from(u32::MAX)
        else {
            panic!("Expected EvaluationExprKind::Numeric and Precedence::NumericOrSymbolicVariable")
        };
        assert_eq!(value, 199);

        let EvaluatedExpr {
            kind: EvaluatedExprKind::Numeric(value),
            precedence: Precedence::NumericOrSymbolicVariable,
        } = EvaluatedExpr::from(1u32) - EvaluatedExpr::from(2u32)
        else {
            panic!("Expected EvaluationExprKind::Numeric and Precedence::NumericOrSymbolicVariable")
        };
        assert_eq!(value, u32::MAX);

        let EvaluatedExpr {
            kind: EvaluatedExprKind::Numeric(value),
            precedence: Precedence::NumericOrSymbolicVariable,
        } = EvaluatedExpr::from(3_000_000_000) * EvaluatedExpr::from(2)
        else {
            panic!("Expected EvaluationExprKind::Numeric and Precedence::NumericOrSymbolicVariable")
        };
        assert_eq!(value, 1_705_032_704);
    }

    #[test]
    fn test_strong_eval_numeric() {
        let EvaluatedExpr {
            kind: EvaluatedExprKind::Numeric(value),
            precedence: Precedence::NumericOrSymbolicVariable,
        } = EvaluatedExpr::from(1u32) + EvaluatedExpr::from(2u32)
        else {
            panic!("Expected EvaluationExprKind::Numeric and Precedence::NumericOrSymbolicVariable")
        };
        assert_eq!(value, 3);

        let EvaluatedExpr {
            kind: EvaluatedExprKind::Numeric(value),
            precedence: Precedence::NumericOrSymbolicVariable,
        } = EvaluatedExpr::from(1u32) + (EvaluatedExpr::from(2u32) + EvaluatedExpr::from(3u32))
        else {
            panic!("Expected EvaluationExprKind::Numeric and Precedence::NumericOrSymbolicVariable")
        };
        assert_eq!(value, 6);

        let EvaluatedExpr {
            kind: EvaluatedExprKind::Numeric(value),
            precedence: Precedence::NumericOrSymbolicVariable,
        } = EvaluatedExpr::from(1u32) + EvaluatedExpr::from(2u32) * EvaluatedExpr::from(3u32)
        else {
            panic!("Expected EvaluationExprKind::Numeric and Precedence::NumericOrSymbolicVariable")
        };
        assert_eq!(value, 7);

        let EvaluatedExpr {
            kind: EvaluatedExprKind::Numeric(value),
            precedence: Precedence::NumericOrSymbolicVariable,
        } = (EvaluatedExpr::from(1u32) + EvaluatedExpr::from(2u32)) * EvaluatedExpr::from(3u32)
        else {
            panic!("Expected EvaluationExprKind::Numeric and Precedence::NumericOrSymbolicVariable")
        };
        assert_eq!(value, 9);

        let EvaluatedExpr {
            kind: EvaluatedExprKind::Numeric(value),
            precedence: Precedence::NumericOrSymbolicVariable,
        } = EvaluatedExpr::from(1u32) * EvaluatedExpr::from(2u32) + EvaluatedExpr::from(3u32)
        else {
            panic!("Expected EvaluationExprKind::Numeric and Precedence::NumericOrSymbolicVariable")
        };
        assert_eq!(value, 5);

        let EvaluatedExpr {
            kind: EvaluatedExprKind::Numeric(value),
            precedence: Precedence::NumericOrSymbolicVariable,
        } = EvaluatedExpr::from(1u32) * (EvaluatedExpr::from(2u32) + EvaluatedExpr::from(3u32))
        else {
            panic!("Expected EvaluationExprKind::Numeric and Precedence::NumericOrSymbolicVariable")
        };
        assert_eq!(value, 5);

        let EvaluatedExpr {
            kind: EvaluatedExprKind::Numeric(value),
            precedence: Precedence::NumericOrSymbolicVariable,
        } = EvaluatedExpr::from(9u32) * EvaluatedExpr::from(10u32)
            - ((EvaluatedExpr::from(7u32) + EvaluatedExpr::from(8u32))
                - (EvaluatedExpr::from(6u32) * EvaluatedExpr::from(3u32)))
        else {
            panic!("Expected EvaluationExprKind::Numeric and Precedence::NumericOrSymbolicVariable")
        };
        assert_eq!(value, 93);
    }

    #[test]
    fn test_strong_eval_symbolic_variables() {
        let EvaluatedExpr {
            kind: EvaluatedExprKind::Value(value),
            precedence: Precedence::Sub,
        } = EvaluatedExpr::from("A") * EvaluatedExpr::from("B")
            - ((EvaluatedExpr::from("C") + EvaluatedExpr::from("D"))
                - (EvaluatedExpr::from("E") * EvaluatedExpr::from(12)))
        else {
            panic!("Expected EvaluationExprKind::Value and Precedence::Sub")
        };
        assert_eq!(value, "A * B - (C + D - E * 12)");
    }

    #[test]
    fn test_strong_eval_add_sub() {
        let EvaluatedExpr {
            kind: EvaluatedExprKind::Value(value),
            precedence: Precedence::Sub,
        } = EvaluatedExpr::from(1) + EvaluatedExpr::from(2) - EvaluatedExpr::from("A")
        else {
            panic!("Expected EvaluationExprKind::Value and Precedence::Sub")
        };
        assert_eq!(value, "3 - A");

        let EvaluatedExpr {
            kind: EvaluatedExprKind::Value(value),
            precedence: Precedence::Add,
        } = EvaluatedExpr::from(1) + (EvaluatedExpr::from(2) - EvaluatedExpr::from("A"))
        else {
            panic!("Expected EvaluationExprKind::Value and Precedence::Add")
        };
        assert_eq!(value, "1 + 2 - A");

        let EvaluatedExpr {
            kind: EvaluatedExprKind::Numeric(value),
            precedence: Precedence::NumericOrSymbolicVariable,
        } = EvaluatedExpr::from(4) + EvaluatedExpr::from(2) - EvaluatedExpr::from(1)
        else {
            panic!("Expected EvaluationExprKind::Numeric and Precedence::NumericOrSymbolicVariable")
        };
        assert_eq!(value, 5);

        let EvaluatedExpr {
            kind: EvaluatedExprKind::Numeric(value),
            precedence: Precedence::NumericOrSymbolicVariable,
        } = EvaluatedExpr::from(4) + (EvaluatedExpr::from(2) - EvaluatedExpr::from(1))
        else {
            panic!("Expected EvaluationExprKind::Numeric and Precedence::NumericOrSymbolicVariable")
        };
        assert_eq!(value, 5);

        let EvaluatedExpr {
            kind: EvaluatedExprKind::Value(value),
            precedence: Precedence::Sub,
        } = EvaluatedExpr::from("A") + EvaluatedExpr::from("B") - EvaluatedExpr::from("C")
        else {
            panic!("Expected EvaluationExprKind::Value and Precedence::Sub")
        };
        assert_eq!(value, "A + B - C");

        let EvaluatedExpr {
            kind: EvaluatedExprKind::Value(value),
            precedence: Precedence::Add,
        } = EvaluatedExpr::from("A") + (EvaluatedExpr::from("B") - EvaluatedExpr::from("C"))
        else {
            panic!("Expected EvaluationExprKind::Value and Precedence::Add")
        };
        assert_eq!(value, "A + B - C");
    }
}
