use std::cmp::Ordering;

use super::types::{Addr, Const, Reg, Value};

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
/// memory address
#[derive(Debug)]
pub(super) enum OperationOutput {
    WriteToRegister(Reg, Value),
    WriteToMemory(Addr, Value),
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

#[derive(Debug, Eq)]
pub(super) struct InflightOperation {
    /// Operation output when it completes
    output: OperationOutput,
    /// Cycle when the operation completes
    complete_cycle: usize,
}

impl Ord for InflightOperation {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.complete_cycle.cmp(&other.complete_cycle) {
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
        self.complete_cycle == other.complete_cycle && self.output == other.output
    }
}

impl InflightOperation {
    /// Load a 32-bit numeric constant into a register
    ///
    /// # Arguments
    /// * `cycle` - cycle when the operation starts
    /// * `dst` - destination register
    /// * `constant` - constant to load
    pub fn from_ldi(cycle: usize, dst: Reg, constant: Const) -> Self {
        Self {
            output: OperationOutput::WriteToRegister(dst, Value(constant.0.to_string())),
            complete_cycle: cycle + OperationLatency::LDI,
        }
    }

    /// Load a value from memory into a register
    ///
    /// # Arguments
    /// * `cycle` - cycle when the operation starts
    /// * `dst` - destination register
    /// * `addr_value` - value of the memory address to load from
    pub fn from_ldr(cycle: usize, dst: Reg, addr_value: Value) -> Self {
        Self {
            output: OperationOutput::WriteToRegister(dst, addr_value),
            complete_cycle: cycle + OperationLatency::LDR,
        }
    }

    /// Store a value from a register into memory
    ///
    /// # Arguments
    /// * `cycle` - cycle when the operation starts
    /// * `src_value` - value of the source register
    /// * `addr` - memory address to store into
    pub fn from_str(cycle: usize, src_value: Value, addr: Addr) -> Self {
        Self {
            output: OperationOutput::WriteToMemory(addr, src_value),
            complete_cycle: cycle + OperationLatency::STR,
        }
    }

    /// Add the values in the source registers and put the sum in the
    /// destination register
    ///
    /// # Arguments
    /// * `cycle` - cycle when the operation starts
    /// * `dst` - destination register
    /// * `src1_value` - value of the first source register
    /// * `src2_value` - value of the second source register
    pub fn from_add(cycle: usize, dst: Reg, src1_value: Value, src2_value: Value) -> Self {
        Self {
            output: OperationOutput::WriteToRegister(
                dst,
                Value(format!("({} + {})", src1_value.0, src2_value.0)),
            ),
            complete_cycle: cycle + OperationLatency::ADD,
        }
    }

    /// Subtract the value of source register 2 from source register 1 and put
    /// the difference in the destination register
    ///
    /// # Arguments
    /// * `cycle` - cycle when the operation starts
    /// * `dst` - destination register
    /// * `src1_value` - value of the first source register
    /// * `src2_value` - value of the second source register
    pub fn from_sub(cycle: usize, dst: Reg, src1_value: Value, src2_value: Value) -> Self {
        Self {
            output: OperationOutput::WriteToRegister(
                dst,
                Value(format!("({} - {})", src1_value.0, src2_value.0)),
            ),
            complete_cycle: cycle + OperationLatency::SUB,
        }
    }

    /// Multiply the values in the source registers and put the product in the
    /// destination register
    ///
    /// # Arguments
    /// * `cycle` - cycle when the operation starts
    /// * `dst` - destination register
    /// * `src1_value` - value of the first source register
    /// * `src2_value` - value of the second source register
    pub fn from_mul(cycle: usize, dst: Reg, src1_value: Value, src2_value: Value) -> Self {
        Self {
            output: OperationOutput::WriteToRegister(
                dst,
                Value(format!("({} * {})", src1_value.0, src2_value.0)),
            ),
            complete_cycle: cycle + OperationLatency::MUL,
        }
    }
}
