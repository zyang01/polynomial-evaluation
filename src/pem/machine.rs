use std::collections::{BinaryHeap, HashMap};

use thiserror::Error;

use super::{
    inflight_operation::{InflightOperation, OperationOutput},
    types::{Addr, Reg, Value},
    Instruction,
};

const REGISTER_COUNT: usize = 8;

#[derive(Debug)]
pub(crate) struct Machine {
    /// Registers
    regs: Vec<Option<Value>>,
    /// Memory
    mem: HashMap<Addr, Value>,
    /// Program counter
    pc: usize,
    pending_operations: BinaryHeap<InflightOperation>,
}

#[derive(Debug, Error)]
pub enum ComputeError {
    #[error("Machine terminated. Please use a new machine.")]
    Terminated,
    #[error("Invalid register #{} at instruction #{pc}", .reg.0)]
    InvalidRegister { reg: Reg, pc: usize },
    #[error("Accessing uninitialized register #{} at instruction #{pc}", .reg.0)]
    UninitializedRegister { reg: Reg, pc: usize },
    #[error("Accessing uninitialized memory address #{} at instruction #{pc}", .addr.0)]
    UninitializedMemory { addr: Addr, pc: usize },
    #[error("Register #{} data race detected at cycle #{pc} originated by instructions #{inst1} and #{inst2}", .reg.0)]
    RegisterDataRace {
        reg: Reg,
        pc: usize,
        inst1: usize,
        inst2: usize,
    },
    #[error("Memory #{} data race detected at cycle #{pc} originated by instructions #{inst1} and #{inst2}", .addr.0)]
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
    pub fn new(mem: HashMap<Addr, Value>) -> Self {
        Self {
            regs: vec![None; REGISTER_COUNT],
            mem,
            pc: 0,
            pending_operations: BinaryHeap::new(),
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
    pub fn compute(&mut self, program: &Vec<Instruction>) -> Result<Value, ComputeError> {
        if self.pc != 0 {
            return Err(ComputeError::Terminated);
        }

        for instruction in program {
            self.begin_execution(instruction)?;
            self.pc += 1;
            self.end_cycle()?;
        }

        while self.pending_operations.peek().is_some() {
            self.pc += 1;
            self.end_cycle()?;
        }

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
    fn get_register_value(&self, reg: Reg) -> Result<Value, ComputeError> {
        self.regs
            .get(reg.0 as usize)
            .ok_or(ComputeError::InvalidRegister { reg, pc: self.pc })?
            .as_ref()
            .ok_or(ComputeError::UninitializedRegister { reg, pc: self.pc })
            .map(|v| v.clone())
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
    fn get_address_value(&self, addr: &Addr) -> Result<Value, ComputeError> {
        self.mem
            .get(addr)
            .ok_or(ComputeError::UninitializedMemory {
                addr: *addr,
                pc: self.pc,
            })
            .map(|v| v.clone())
    }

    /// Begin execution of an instruction by creating an `InflightOperation`
    /// for each operation in the instruction and reading the values of the
    /// source registers or memory addresses
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
    ///  same register
    /// * `Err(ComputeError::MemoryDataRace)` if two operations wrote to the
    /// same memory address
    ///
    /// # Panics
    /// * If the `complete_cycle` of an `InflightOperation` is less than the
    ///  `pc` of the `Machine`
    fn end_cycle(&mut self) -> Result<(), ComputeError> {
        let mut prev = None;
        while let Some(next) = self.pending_operations.peek() {
            let complete_cycle = next.get_complete_cycle();
            assert!(complete_cycle >= self.pc);

            if complete_cycle > self.pc {
                break;
            }

            let next = self.pending_operations.pop().unwrap();
            let output = next.get_output();
            if prev.as_ref() == Some(&next) {
                return Err(match output {
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
                });
            }

            match output {
                OperationOutput::WriteToRegister(reg, value) => {
                    self.regs[reg.0 as usize] = Some(value.clone());
                }
                OperationOutput::WriteToMemory(addr, value) => {
                    self.mem.insert(*addr, value.clone());
                }
            }

            prev = Some(next);
        }

        Ok(())
    }
}
