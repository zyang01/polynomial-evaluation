use super::types::{Addr, Const, Reg};

#[derive(Debug)]
pub(crate) struct Instruction {
    /// ldi <reg> <const> - load a 32-bit numeric constant into a register
    pub(super) ldi: Option<(Reg, Const)>,
    /// ldr <reg> <addr> - load value from memory into a register
    pub(super) ldr: Option<(Reg, Addr)>,
    /// str <Reg> <Addr> - store a value from register into memory
    pub(super) str: Option<(Reg, Addr)>,
    /// add <dst> <src1> <src2> - add the values in the source registers and put
    /// the sum in the destination register
    pub(super) add: Option<(Reg, Reg, Reg)>,
    /// sub <dst> <src1> <src2> - subtract the value of source register 2 from
    /// source register 1 and put the difference in the destination register
    pub(super) sub: Option<(Reg, Reg, Reg)>,
    /// mul <dst> <src1> <src2> - multiply the values in the source registers
    /// and put the product in the destination register
    pub(super) mul: Option<(Reg, Reg, Reg)>,
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;

        if let Some((reg, constant)) = &self.ldi {
            write!(f, " ldi {} {};", reg, constant)?;
        }

        if let Some((reg, addr)) = &self.ldr {
            write!(f, " ldr {} {};", reg, addr)?;
        }

        if let Some((reg, addr)) = &self.str {
            write!(f, " str {} {};", reg, addr)?;
        }

        if let Some((dst, src1, src2)) = &self.add {
            write!(f, " add {} {} {};", dst, src1, src2)?;
        }

        if let Some((dst, src1, src2)) = &self.sub {
            write!(f, " sub {} {} {};", dst, src1, src2)?;
        }

        if let Some((dst, src1, src2)) = &self.mul {
            write!(f, " mul {} {} {};", dst, src1, src2)?;
        }

        write!(f, " }}")?;

        Ok(())
    }
}

impl Instruction {
    /// Create an empty `Instruction`
    pub fn new() -> Self {
        Self {
            ldi: None,
            ldr: None,
            str: None,
            add: None,
            sub: None,
            mul: None,
        }
    }

    /// Set `ldi` instruction to load a constant into a register
    ///
    /// # Arguments
    /// * `dst` - destination register
    /// * `constant` - constant to load
    pub fn with_ldi(mut self, dst: Reg, constant: Const) -> Self {
        self.ldi = Some((dst, constant));
        self
    }

    /// Set `ldr`` instruction to load a value from memory into a register
    ///
    /// # Arguments
    /// * `dst` - destination register
    /// * `addr` - memory address to load from
    pub fn with_ldr(mut self, dst: Reg, addr: Addr) -> Self {
        self.ldr = Some((dst, addr));
        self
    }

    /// Set `str`` instruction to store a value from a register into memory
    ///
    /// # Arguments
    /// * `src` - source register
    /// * `addr` - memory address to store into
    pub fn with_str(mut self, src: Reg, addr: Addr) -> Self {
        self.str = Some((src, addr));
        self
    }

    /// Set `add` instruction to add the values in the source registers and put
    /// the sum in the destination register
    ///
    /// # Note
    /// `add dst src1 src2` is evaluated as `dst = src2 + src1`
    ///
    /// # Arguments
    /// * `dst` - destination register
    /// * `src1` - source register 1
    /// * `src2` - source register 2
    pub fn with_add(mut self, dst: Reg, src1: Reg, src2: Reg) -> Self {
        self.add = Some((dst, src1, src2));
        self
    }

    /// Set `sub` instruction to subtract the value of source register 2 from
    /// source register 1 and put the difference in the destination register
    ///
    /// # Arguments
    /// * `dst` - destination register
    /// * `src1` - source register 1
    /// * `src2` - source register 2
    pub fn with_sub(mut self, dst: Reg, src1: Reg, src2: Reg) -> Self {
        self.sub = Some((dst, src1, src2));
        self
    }

    /// Set `mul` instruction to multiply the values in the source registers and
    /// put the product in the destination register
    ///
    /// # Arguments
    /// * `dst` - destination register
    /// * `src1` - source register 1
    /// * `src2` - source register 2
    pub fn with_mul(mut self, dst: Reg, src1: Reg, src2: Reg) -> Self {
        self.mul = Some((dst, src1, src2));
        self
    }
}
