use std::cmp::Ordering;

use log::trace;

use super::{
    types::{Addr, Const, Reg},
    ExprWrapper,
};

struct OperationLatency;
impl OperationLatency {
    const LDI: usize = 1;
    const LDR: usize = 5;
    const STR: usize = 5;
    const ADD: usize = 2;
    const SUB: usize = 2;
    const MUL: usize = 10;
}

/// Output of an operation
///
/// We consider two operations to be equal if they write to the same register or
/// memory address and the ordering of operations is determined by the tuple
/// `(output_type, register/memory_address)`. Two outputs writing to the same
/// register/memory address are considered equal regardless of the values.
#[derive(Debug)]
pub(super) enum OperationOutput {
    WriteToRegister(Reg, ExprWrapper),
    WriteToMemory(Addr, ExprWrapper),
}

impl std::fmt::Display for OperationOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WriteToRegister(dst, value) => write!(f, "WriteToRegister({}, `{}`)", dst, value),
            Self::WriteToMemory(addr, value) => write!(f, "WriteToMemory({}, `{}`)", addr, value),
        }
    }
}

impl Ord for OperationOutput {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::WriteToRegister(dst1, _), Self::WriteToRegister(dst2, _)) => dst1.0.cmp(&dst2.0),
            (Self::WriteToMemory(addr1, _), Self::WriteToMemory(addr2, _)) => addr1.0.cmp(&addr2.0),
            (Self::WriteToRegister(_, _), Self::WriteToMemory(_, _)) => Ordering::Less,
            (Self::WriteToMemory(_, _), Self::WriteToRegister(_, _)) => Ordering::Greater,
        }
    }
}

impl PartialOrd for OperationOutput {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for OperationOutput {}

impl PartialEq for OperationOutput {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::WriteToRegister(dst1, _), Self::WriteToRegister(dst2, _)) => dst1.0 == dst2.0,
            (Self::WriteToMemory(addr1, _), Self::WriteToMemory(addr2, _)) => addr1.0 == addr2.0,
            _ => false,
        }
    }
}

/// An operation that is currently in-flight and will complete just before
/// cycle `complete_by` starts, with OperationOutput `output`.
///
/// We consider two in-flight operations to be equal if they complete at the
/// same cycle and have the same output.
#[derive(Debug, Eq)]
pub(super) struct InflightOperation {
    /// Operation output when it completes
    output: OperationOutput,
    /// Cycle by which the operation completes
    complete_by: usize,
    /// The cycle when the operation started
    started_at: usize,
}

impl Ord for InflightOperation {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.complete_by.cmp(&other.complete_by) {
            Ordering::Equal => self.output.cmp(&other.output),
            ordering => ordering.reverse(),
        }
    }
}

impl PartialOrd for InflightOperation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for InflightOperation {
    fn eq(&self, other: &Self) -> bool {
        self.complete_by == other.complete_by && self.output == other.output
    }
}

impl InflightOperation {
    /// Load a 32-bit numeric constant into a register
    ///
    /// # Arguments
    /// * `cycle` - cycle when the operation starts
    /// * `dst` - destination register
    /// * `constant` - constant to load
    pub fn from_ldi(cycle: usize, dst: Reg, Const(constant): Const) -> Self {
        let myself = Self {
            output: OperationOutput::WriteToRegister(dst, constant.into()),
            complete_by: cycle + OperationLatency::LDI,
            started_at: cycle,
        };
        trace!(
            "LDI operation started at cycle #{} and expect to complete by cycle #{}",
            cycle,
            myself.complete_by
        );
        myself
    }

    /// Load a value from memory into a register
    ///
    /// # Arguments
    /// * `cycle` - cycle when the operation starts
    /// * `dst` - destination register
    /// * `addr_value` - value of the memory address to load from
    pub fn from_ldr(cycle: usize, dst: Reg, addr_value: &ExprWrapper) -> Self {
        let myself = Self {
            output: OperationOutput::WriteToRegister(dst, addr_value.clone()),
            complete_by: cycle + OperationLatency::LDR,
            started_at: cycle,
        };
        trace!(
            "LDR operation started at cycle #{} and expect to complete by cycle #{}",
            cycle,
            myself.complete_by
        );
        myself
    }

    /// Store a value from a register into memory
    ///
    /// # Arguments
    /// * `cycle` - cycle when the operation starts
    /// * `src_value` - value of the source register
    /// * `addr` - memory address to store into
    pub fn from_str(cycle: usize, src_value: &ExprWrapper, addr: Addr) -> Self {
        let myself = Self {
            output: OperationOutput::WriteToMemory(addr, src_value.clone()),
            complete_by: cycle + OperationLatency::STR,
            started_at: cycle,
        };
        trace!(
            "STR operation started at cycle #{} and expect to complete by cycle #{}",
            cycle,
            myself.complete_by
        );
        myself
    }

    /// Add the values in the source registers and put the sum in the
    /// destination register
    ///
    /// # Note
    /// `add dst src1 src2` is evaluated as `dst = src2 + src1`
    ///
    /// # Arguments
    /// * `cycle` - cycle when the operation starts
    /// * `dst` - destination register
    /// * `src1_value` - value of the first source register
    /// * `src2_value` - value of the second source register
    pub fn from_add(
        cycle: usize,
        dst: Reg,
        src1_value: &ExprWrapper,
        src2_value: &ExprWrapper,
    ) -> Self {
        let myself = Self {
            // NB src2 is lhs and src1 is rhs
            output: OperationOutput::WriteToRegister(dst, src2_value + src1_value),
            complete_by: cycle + OperationLatency::ADD,
            started_at: cycle,
        };
        trace!(
            "ADD operation started at cycle #{} and expect to complete by cycle #{}",
            cycle,
            myself.complete_by
        );
        myself
    }

    /// Subtract the value of source register 2 from source register 1 and put
    /// the difference in the destination register
    ///
    /// # Arguments
    /// * `cycle` - cycle when the operation starts
    /// * `dst` - destination register
    /// * `src1_value` - value of the first source register
    /// * `src2_value` - value of the second source register
    pub fn from_sub(
        cycle: usize,
        dst: Reg,
        src1_value: &ExprWrapper,
        src2_value: &ExprWrapper,
    ) -> Self {
        let myself = Self {
            output: OperationOutput::WriteToRegister(dst, src1_value - src2_value),
            complete_by: cycle + OperationLatency::SUB,
            started_at: cycle,
        };
        trace!(
            "SUB operation started at cycle #{} and expect to complete by cycle #{}",
            cycle,
            myself.complete_by
        );
        myself
    }

    /// Multiply the values in the source registers and put the product in the
    /// destination register
    ///
    /// # Arguments
    /// * `cycle` - cycle when the operation starts
    /// * `dst` - destination register
    /// * `src1_value` - value of the first source register
    /// * `src2_value` - value of the second source register
    pub fn from_mul(
        cycle: usize,
        dst: Reg,
        src1_value: &ExprWrapper,
        src2_value: &ExprWrapper,
    ) -> Self {
        let myself = Self {
            output: OperationOutput::WriteToRegister(dst, src1_value * src2_value),
            complete_by: cycle + OperationLatency::MUL,
            started_at: cycle,
        };
        trace!(
            "MUL operation started at cycle #{} and expect to complete by cycle #{}",
            cycle,
            myself.complete_by
        );
        myself
    }

    pub fn get_output(&self) -> &OperationOutput {
        &self.output
    }

    pub fn get_complete_by(&self) -> usize {
        self.complete_by
    }

    pub fn get_instruction(&self) -> usize {
        self.started_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_output_equality() {
        let op1 = OperationOutput::WriteToRegister(Reg(0), 1.into());
        let op2 = OperationOutput::WriteToRegister(Reg(0), 2.into());
        let op3 = OperationOutput::WriteToMemory(Addr(0), 3.into());
        let op4 = OperationOutput::WriteToMemory(Addr(0), 4.into());

        assert_eq!(op1, op1);
        assert_eq!(op1, op2);
        assert_ne!(op1, op3);
        assert_ne!(op1, op4);

        assert_eq!(op2, op2);
        assert_ne!(op2, op3);
        assert_ne!(op2, op4);

        assert_eq!(op3, op3);
        assert_eq!(op3, op4);

        assert_eq!(op4, op4);
    }

    #[test]
    fn test_inflight_operation_ordering() {
        let ldi = InflightOperation::from_ldi(0, Reg(0), Const(1));
        let ldr = InflightOperation::from_ldr(0, Reg(0), &1.into());
        let str = InflightOperation::from_str(0, &1.into(), Addr(0));
        let add = InflightOperation::from_add(0, Reg(0), &1.into(), &2.into());
        let sub = InflightOperation::from_sub(0, Reg(0), &1.into(), &2.into());
        let mul = InflightOperation::from_mul(0, Reg(0), &1.into(), &2.into());

        assert_eq!(ldi, ldi);
        assert_eq!(ldr, ldr);
        assert_eq!(str, str);
        assert_eq!(add, add);
        assert_eq!(sub, sub);
        assert_eq!(mul, mul);

        assert!(ldi > ldr);
        assert!(ldi > str);
        assert!(ldi > add);
        assert!(ldi > sub);
        assert!(ldi > mul);

        assert!(ldr < str);
        assert!(ldr < add);
        assert!(ldr < sub);
        assert!(ldr > mul);

        assert!(str < add);
        assert!(str < sub);
        assert!(str > mul);

        assert!(add == sub);
        assert!(add > mul);

        assert!(sub > mul)
    }

    #[test]
    fn test_inflight_operation_ldi() {
        let ldi = InflightOperation::from_ldi(0, Reg(0), Const(1));
        let OperationOutput::WriteToRegister(reg, value) = ldi.get_output() else {
            panic!("Expected WriteToRegister, got {:?}", ldi.get_output());
        };
        assert_eq!(*reg, Reg(0));
        assert_eq!(value.weak_eval(), String::from("1"));
        assert_eq!(ldi.get_complete_by(), OperationLatency::LDI);
        assert_eq!(ldi.get_instruction(), 0);
    }

    #[test]
    fn test_inflight_operation_ldr() {
        let ldr = InflightOperation::from_ldr(0, Reg(0), &1.into());
        let OperationOutput::WriteToRegister(reg, value) = ldr.get_output() else {
            panic!("Expected WriteToRegister, got {:?}", ldr.get_output());
        };
        assert_eq!(*reg, Reg(0));
        assert_eq!(value.weak_eval(), String::from("1"));
        assert_eq!(ldr.get_complete_by(), OperationLatency::LDR);
        assert_eq!(ldr.get_instruction(), 0);
    }

    #[test]
    fn test_inflight_operation_str() {
        let str_ = InflightOperation::from_str(0, &1.into(), Addr(0));
        let OperationOutput::WriteToMemory(addr, value) = str_.get_output() else {
            panic!("Expected WriteToMemory, got {:?}", str_.get_output());
        };
        assert_eq!(*addr, Addr(0));
        assert_eq!(value.weak_eval(), String::from("1"));
        assert_eq!(str_.get_complete_by(), OperationLatency::STR);
        assert_eq!(str_.get_instruction(), 0);
    }

    #[test]
    fn test_inflight_operation_add() {
        let add = InflightOperation::from_add(0, Reg(0), &1.into(), &2.into());
        let OperationOutput::WriteToRegister(reg, value) = add.get_output() else {
            panic!("Expected WriteToRegister, got {:?}", add.get_output());
        };
        assert_eq!(*reg, Reg(0));
        assert_eq!(value.weak_eval(), String::from("(2 + 1)"));
        assert_eq!(value.strong_eval(), String::from("3"));
        assert_eq!(add.get_complete_by(), OperationLatency::ADD);
        assert_eq!(add.get_instruction(), 0);
    }

    #[test]
    fn test_inflight_operation_sub() {
        let sub = InflightOperation::from_sub(0, Reg(0), &1.into(), &2.into());
        let OperationOutput::WriteToRegister(reg, value) = sub.get_output() else {
            panic!("Expected WriteToRegister, got {:?}", sub.get_output());
        };
        assert_eq!(*reg, Reg(0));
        assert_eq!(value.weak_eval(), String::from("(1 - 2)"));
        assert_eq!(value.strong_eval(), String::from("4294967295"));
        assert_eq!(sub.get_complete_by(), OperationLatency::SUB);
        assert_eq!(sub.get_instruction(), 0);
    }

    #[test]
    fn test_inflight_operation_mul() {
        let mul = InflightOperation::from_mul(0, Reg(0), &1.into(), &2.into());
        let OperationOutput::WriteToRegister(reg, value) = mul.get_output() else {
            panic!("Expected WriteToRegister, got {:?}", mul.get_output());
        };
        assert_eq!(*reg, Reg(0));
        assert_eq!(value.weak_eval(), String::from("(1 * 2)"));
        assert_eq!(value.strong_eval(), String::from("2"));
        assert_eq!(mul.get_complete_by(), OperationLatency::MUL);
        assert_eq!(mul.get_instruction(), 0);
    }
}
