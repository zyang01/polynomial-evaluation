use std::collections::{BinaryHeap, HashMap};

use log::{debug, trace, warn};
use thiserror::Error;

use super::{
    inflight_operation::{InflightOperation, OperationOutput},
    types::{Addr, Reg},
    ExprWrapper, Instruction,
};

const REGISTER_COUNT: usize = 8;

/// Polynomial Evaluation Machine (PEM) with 8 32-bit registers and a 32-bit
/// addressable memory
#[derive(Debug)]
pub(crate) struct Machine {
    /// Registers
    regs: Vec<Option<ExprWrapper>>,
    /// Memory
    mem: HashMap<Addr, ExprWrapper>,
    /// Program counter
    pc: usize,
    /// Pending operations
    pending_operations: BinaryHeap<InflightOperation>,

    allow_data_race: bool,
}

#[derive(Debug, Error, PartialEq)]
pub enum ComputeError {
    #[error("Machine terminated. Please use a new machine.")]
    Terminated,
    #[error("Invalid register #{} at instruction #{pc}", .reg.0)]
    InvalidRegister { reg: Reg, pc: usize },
    #[error("Accessing uninitialized register #{} at instruction #{pc}", .reg.0)]
    UninitializedRegister { reg: Reg, pc: usize },
    #[error("Accessing uninitialized memory address #{} at instruction #{pc}", .addr.0)]
    UninitializedMemory { addr: Addr, pc: usize },
    #[error("Register #{} data race detected at cycle #{pc} from operations originated by instructions #{inst1} and #{inst2}", .reg.0)]
    RegisterDataRace {
        reg: Reg,
        pc: usize,
        inst1: usize,
        inst2: usize,
    },
    #[error("Memory #{} data race detected at cycle #{pc} from operations originated by instructions #{inst1} and #{inst2}", .addr.0)]
    MemoryDataRace {
        addr: Addr,
        pc: usize,
        inst1: usize,
        inst2: usize,
    },
}

impl Machine {
    /// Initialize a new `Machine` with given memory
    ///
    /// # Arguments
    /// * `mem` - memory to initialize with
    pub fn new(mem: HashMap<Addr, ExprWrapper>) -> Self {
        Self {
            regs: vec![None; REGISTER_COUNT],
            mem,
            pc: 0,
            pending_operations: BinaryHeap::new(),
            allow_data_race: false,
        }
    }

    pub fn allow_data_race(&mut self, allow: bool) {
        self.allow_data_race = allow;
        if allow {
            warn!("Allowing data races \u{1F648}");
        }
    }

    /// Compute the result of a program
    ///
    /// # Arguments
    /// * `program` - a vector of `Instruction`s to compute
    ///
    /// # Returns
    /// * `Ok(value)` if the program terminated successfully
    /// * `Err(ComputeError)` if the program terminated with an error
    pub fn compute(&mut self, program: &Vec<Instruction>) -> Result<&ExprWrapper, ComputeError> {
        if self.pc != 0 {
            return Err(ComputeError::Terminated);
        }

        for instruction in program {
            debug!("Executing instruction #{}: {}", self.pc, instruction);
            self.begin_execution(instruction)?;
            self.end_cycle()?;
        }

        while self.pending_operations.peek().is_some() {
            self.end_cycle()?;
        }

        debug!("All instructions executed");

        self.get_register_value(Reg(0))
    }

    /// Validate a register and return it if valid
    ///
    /// # Arguments
    /// * `reg` - register to validate
    ///
    /// # Returns
    /// * `Ok(reg)` if `reg` is valid
    /// * `Err(ComputeError::InvalidRegister)` if `reg` is invalid
    fn validated_register(&self, reg: Reg) -> Result<Reg, ComputeError> {
        if reg.0 >= REGISTER_COUNT as u32 {
            return Err(ComputeError::InvalidRegister { reg, pc: self.pc });
        }
        Ok(reg)
    }

    /// Get the value of a register
    ///
    /// # Arguments
    /// * `reg` - register to get the value of
    ///
    /// # Returns
    /// * `Ok(value)` if the register is valid and initialized
    /// * `Err(ComputeError::InvalidRegister)` if the register is invalid
    /// * `Err(ComputeError::UninitializedRegister)` if the register is valid
    ///   but uninitialized
    fn get_register_value(&self, reg: Reg) -> Result<&ExprWrapper, ComputeError> {
        self.regs
            .get(reg.0 as usize)
            .ok_or(ComputeError::InvalidRegister { reg, pc: self.pc })?
            .as_ref()
            .ok_or(ComputeError::UninitializedRegister { reg, pc: self.pc })
            .map(|v| {
                trace!(
                    "Register {} accessed with value `{}` at cycle #{}",
                    reg,
                    v,
                    self.pc
                );
                v
            })
    }

    /// Get the value of a memory address
    ///
    /// # Arguments
    /// * `addr` - memory address to get the value of
    ///
    /// # Returns
    /// * `Ok(value)` if the memory address is valid and initialized
    /// * `Err(ComputeError::UninitializedMemory)` if the memory address is
    ///   valid but uninitialized
    fn get_address_value(&self, addr: &Addr) -> Result<&ExprWrapper, ComputeError> {
        self.mem
            .get(addr)
            .ok_or(ComputeError::UninitializedMemory {
                addr: *addr,
                pc: self.pc,
            })
            .map(|v| {
                trace!(
                    "Memory address {} accessed with value `{}` at cycle #{}",
                    addr,
                    v,
                    self.pc
                );
                v
            })
    }

    /// Begin execution of an instruction by reading operands from registers or
    /// memory, and create an `InflightOperation` for each operation
    ///
    /// # Arguments
    /// * `instruction` - instruction to execute
    ///
    /// # Returns
    /// * `Ok(())` if the instruction execution was successfully started
    /// * `Err(ComputeError)` if the instruction execution failed
    fn begin_execution(&mut self, instruction: &Instruction) -> Result<(), ComputeError> {
        if let Some((dst, constant)) = instruction.ldi {
            self.pending_operations.push(InflightOperation::from_ldi(
                self.pc,
                self.validated_register(dst)?,
                constant,
            ));
        }

        if let Some((dst, addr)) = instruction.ldr {
            self.pending_operations.push(InflightOperation::from_ldr(
                self.pc,
                self.validated_register(dst)?,
                self.get_address_value(&addr)?,
            ));
        }

        if let Some((src, addr)) = instruction.str {
            self.pending_operations.push(InflightOperation::from_str(
                self.pc,
                self.get_register_value(src)?,
                addr,
            ));
        }

        if let Some((dst, src1, src2)) = instruction.add {
            self.pending_operations.push(InflightOperation::from_add(
                self.pc,
                self.validated_register(dst)?,
                self.get_register_value(src1)?,
                self.get_register_value(src2)?,
            ));
        }

        if let Some((dst, src1, src2)) = instruction.sub {
            self.pending_operations.push(InflightOperation::from_sub(
                self.pc,
                self.validated_register(dst)?,
                self.get_register_value(src1)?,
                self.get_register_value(src2)?,
            ));
        }

        if let Some((dst, src1, src2)) = instruction.mul {
            self.pending_operations.push(InflightOperation::from_mul(
                self.pc,
                self.validated_register(dst)?,
                self.get_register_value(src1)?,
                self.get_register_value(src2)?,
            ));
        }

        Ok(())
    }

    /// End a cycle by writing the output of all completed operations to
    /// registers or memory
    ///
    /// # Returns
    /// * `Ok(())` if the cycle was successfully ended
    /// * `Err(ComputeError::RegisterDataRace)` if two operations wrote to the
    ///   same register and `allow_data_race` is `false`
    /// * `Err(ComputeError::MemoryDataRace)` if two operations wrote to the
    ///   same memory address and `allow_data_race` is `false`
    ///
    /// # Panics
    /// * If the `complete_by` of an `InflightOperation` is less than or equal
    ///   to the program counter `pc`
    fn end_cycle(&mut self) -> Result<(), ComputeError> {
        let mut prev: Option<InflightOperation> = None;
        while let Some(next) = self.pending_operations.peek() {
            let complete_by = next.get_complete_by();
            assert!(complete_by > self.pc);

            if complete_by > self.pc + 1 {
                break;
            }

            let next = self.pending_operations.pop().unwrap();
            let output = next.get_output();
            debug!(
                "Operation originated by instruction #{} completed at cycle #{}: {}",
                next.get_instruction(),
                self.pc,
                output
            );

            if prev.as_ref().map(|op| op.get_output()) == Some(output) {
                let err = match output {
                    OperationOutput::WriteToRegister(reg, _) => ComputeError::RegisterDataRace {
                        reg: *reg,
                        pc: self.pc,
                        inst1: prev.unwrap().get_instruction(),
                        inst2: next.get_instruction(),
                    },
                    OperationOutput::WriteToMemory(addr, _) => ComputeError::MemoryDataRace {
                        addr: *addr,
                        pc: self.pc,
                        inst1: prev.unwrap().get_instruction(),
                        inst2: next.get_instruction(),
                    },
                };

                if !self.allow_data_race {
                    return Err(err);
                }
                warn!("{err}");
            }

            match output {
                OperationOutput::WriteToRegister(reg, value) => {
                    self.regs[reg.0 as usize] = Some(value.clone());
                    trace!(
                        "Register {} written with value `{}` at cycle #{}",
                        reg,
                        value,
                        self.pc
                    )
                }
                OperationOutput::WriteToMemory(addr, value) => {
                    self.mem.insert(*addr, value.clone());
                    trace!(
                        "Memory address {} written with value `{}` at cycle #{}",
                        addr,
                        value,
                        self.pc
                    )
                }
            }

            prev = Some(next);
        }

        debug!("Cycle #{} completed", self.pc);
        self.pc += 1;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::pem::types::{Const, Reg};

    use super::*;

    #[test]
    fn test_str() {
        let mut machine = Machine::new(HashMap::new());
        let program = Vec::from([
            Instruction::new().with_ldi(Reg(0), Const(1)),
            Instruction::new().with_str(Reg(0), Addr(0)),
        ]);
        let expr = machine.compute(&program).unwrap();
        assert_eq!(expr.weak_eval(), "1".to_string());
        assert_eq!(expr.strong_eval(), "1".to_string());
        machine
            .get_address_value(&Addr(0))
            .map(|v| {
                assert_eq!(v.weak_eval(), "1".to_string());
            })
            .expect("Memory address 0 should be initialized");
        assert_eq!(machine.pc, 6);
    }

    #[test]
    fn test_add() {
        let mut machine = Machine::new(HashMap::new());
        let program = Vec::from([
            Instruction::new().with_ldi(Reg(0), Const(1)),
            Instruction::new().with_ldi(Reg(1), Const(8)),
            Instruction::new().with_add(Reg(0), Reg(0), Reg(1)),
        ]);
        let expr = machine.compute(&program).unwrap();
        assert_eq!(expr.weak_eval(), "(8 + 1)".to_string());
        assert_eq!(expr.strong_eval(), "9".to_string());
        assert_eq!(machine.pc, 4);
    }

    #[test]
    fn test_sub() {
        let mut machine = Machine::new(HashMap::new());
        let program = Vec::from([
            Instruction::new().with_ldi(Reg(0), Const(1)),
            Instruction::new().with_ldi(Reg(1), Const(8)),
            Instruction::new().with_sub(Reg(0), Reg(1), Reg(0)),
        ]);
        let expr = machine.compute(&program).unwrap();
        assert_eq!(expr.weak_eval(), "(8 - 1)".to_string());
        assert_eq!(expr.strong_eval(), "7".to_string());
        assert_eq!(machine.pc, 4);
    }

    #[test]
    fn test_mul() {
        let mut machine = Machine::new(HashMap::new());
        let program = Vec::from([
            Instruction::new().with_ldi(Reg(0), Const(2)),
            Instruction::new().with_ldi(Reg(1), Const(8)),
            Instruction::new().with_mul(Reg(0), Reg(0), Reg(1)),
        ]);
        let expr = machine.compute(&program).unwrap();
        assert_eq!(expr.weak_eval(), "(2 * 8)".to_string());
        assert_eq!(expr.strong_eval(), "16".to_string());
        assert_eq!(machine.pc, 12);
    }

    #[test]
    fn test_example_program() {
        let mut machine =
            Machine::new(HashMap::from_iter(('A'..='Z').enumerate().map(|(i, c)| {
                (Addr(i as u32), ExprWrapper::from_symbolic_variable(c))
            })));
        let program = Vec::from([
            Instruction::new()
                .with_ldi(Reg(0), Const(1))
                .with_ldr(Reg(1), Addr(0)),
            Instruction::new()
                .with_ldi(Reg(2), Const(2))
                .with_ldr(Reg(3), Addr(1)),
            Instruction::new(),
            Instruction::new(),
            Instruction::new(),
            Instruction::new().with_add(Reg(0), Reg(0), Reg(1)),
            Instruction::new().with_add(Reg(2), Reg(2), Reg(3)),
            Instruction::new(),
            Instruction::new().with_mul(Reg(0), Reg(0), Reg(2)),
        ]);
        let expr = machine.compute(&program).unwrap();
        assert_eq!(expr.weak_eval(), "((A + 1) * (B + 2))".to_string());
        assert_eq!(expr.strong_eval(), "(A + 1) * (B + 2)".to_string());
        assert_eq!(machine.pc, 18);
    }

    #[test]
    fn test_long_polynomial() {
        let mut machine =
            Machine::new(HashMap::from_iter(('A'..='Z').enumerate().map(|(i, c)| {
                (Addr(i as u32), ExprWrapper::from_symbolic_variable(c))
            })));
        let program = Vec::from([
            Instruction::new()
                .with_ldi(Reg(3), Const(4))
                .with_ldr(Reg(6), Addr(2)),
            Instruction::new()
                .with_ldi(Reg(2), Const(3))
                .with_ldr(Reg(7), Addr(3)),
            Instruction::new()
                .with_ldi(Reg(1), Const(2))
                .with_ldr(Reg(5), Addr(1)),
            Instruction::new()
                .with_ldi(Reg(0), Const(1))
                .with_ldr(Reg(4), Addr(0)),
            Instruction::new(),
            Instruction::new().with_mul(Reg(3), Reg(3), Reg(6)),
            Instruction::new().with_sub(Reg(3), Reg(6), Reg(7)),
            Instruction::new().with_add(Reg(1), Reg(1), Reg(5)),
            Instruction::new().with_add(Reg(0), Reg(0), Reg(4)),
            Instruction::new().with_add(Reg(2), Reg(2), Reg(3)),
            Instruction::new().with_mul(Reg(1), Reg(0), Reg(1)),
            Instruction::new(),
            Instruction::new(),
            Instruction::new(),
            Instruction::new(),
            Instruction::new().with_mul(Reg(3), Reg(3), Reg(7)),
            Instruction::new(),
            Instruction::new(),
            Instruction::new(),
            Instruction::new(),
            Instruction::new().with_mul(Reg(1), Reg(1), Reg(2)),
            Instruction::new(),
            Instruction::new(),
            Instruction::new(),
            Instruction::new(),
            Instruction::new(),
            Instruction::new(),
            Instruction::new(),
            Instruction::new(),
            Instruction::new(),
            Instruction::new().with_add(Reg(0), Reg(3), Reg(1)),
        ]);
        let expr = machine.compute(&program).unwrap();
        assert_eq!(
            expr.weak_eval(),
            "((((A + 1) * (B + 2)) * ((C - D) + 3)) + ((4 * C) * D))".to_string()
        );
        assert_eq!(
            expr.strong_eval(),
            "(A + 1) * (B + 2) * (C - D + 3) + 4 * C * D".to_string()
        );
        assert_eq!(machine.pc, 32);
    }

    #[test]
    fn test_uninitialized_0_register() {
        let mut machine = Machine::new(HashMap::new());
        let program = Vec::new();
        assert!(machine
            .compute(&program)
            .is_err_and(|e| e == ComputeError::UninitializedRegister { reg: Reg(0), pc: 0 }));
    }

    #[test]
    fn test_terminated() {
        let mut machine = Machine::new(HashMap::new());
        let program = Vec::from([Instruction::new().with_ldi(Reg(0), Const(0))]);
        assert!(machine.compute(&program).is_ok());
        assert!(machine
            .compute(&program)
            .is_err_and(|e| e == ComputeError::Terminated));
    }

    #[test]
    fn test_invalid_register() {
        let mut machine = Machine::new(HashMap::new());
        let reg = REGISTER_COUNT as u32;
        let program = Vec::from([Instruction::new().with_ldi(Reg(reg), Const(0))]);
        assert!(machine.compute(&program).is_err_and(|e| e
            == ComputeError::InvalidRegister {
                reg: Reg(reg),
                pc: 0
            }));
    }

    #[test]
    fn test_uninitialized_register() {
        let mut machine = Machine::new(HashMap::new());
        let program = Vec::from([Instruction::new().with_add(Reg(0), Reg(0), Reg(0))]);
        assert!(machine
            .compute(&program)
            .is_err_and(|e| e == ComputeError::UninitializedRegister { reg: Reg(0), pc: 0 }));
    }

    #[test]
    fn test_uninitialized_memory() {
        let mut machine = Machine::new(HashMap::new());
        let program = Vec::from([Instruction::new().with_ldr(Reg(0), Addr(0))]);
        assert!(machine.compute(&program).is_err_and(|e| e
            == ComputeError::UninitializedMemory {
                addr: Addr(0),
                pc: 0
            }));
    }

    #[test]
    fn test_register_data_race() {
        let mut machine = Machine::new(HashMap::new());
        let program = Vec::from([
            Instruction::new().with_ldi(Reg(0), Const(1)),
            Instruction::new().with_add(Reg(1), Reg(0), Reg(0)),
            Instruction::new().with_ldi(Reg(1), Const(3)),
        ]);
        assert!(machine.compute(&program).is_err_and(|e| e
            == ComputeError::RegisterDataRace {
                reg: Reg(1),
                pc: 2,
                inst1: 1,
                inst2: 2
            }));
    }

    #[test]
    fn test_memory_data_race() {
        println!("Memory data race is not possible with the current operation set");
    }
}
