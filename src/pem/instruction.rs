use super::types::{Addr, Const, Reg};

#[derive(Debug)]
pub(crate) struct Instruction {
    // ldi <reg> <const> - load a 32-bit numeric constant into a register
    ldi: Option<(Reg, Const)>,
    // ldr <reg> <addr> - load value from memory into a register
    ldr: Option<(Reg, Addr)>,
    // str <Reg> <Addr> - store a value from register into memory
    str: Option<(Reg, Addr)>,
    // add <dst> <src1> <src2> - add the values in the source registers and put the sum in the
    // destination register
    add: Option<(Reg, Reg, Reg)>,
    // sub <dst> <src1> <src2> - subtract the value of source register 2 from source register 1 and
    // put the difference in the destination register
    sub: Option<(Reg, Reg, Reg)>,
    // mul <dst> <src1> <src2> - multiply the values in the source registers and put the product in
    // the destination register
    mul: Option<(Reg, Reg, Reg)>,
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

    /// Set `add` instruction to add the values in the source registers and put the sum in the
    /// destination register
    ///
    /// # Arguments
    /// * `dst` - destination register
    /// * `src1` - source register 1
    /// * `src2` - source register 2
    pub fn with_add(mut self, dst: Reg, src1: Reg, src2: Reg) -> Self {
        self.add = Some((dst, src1, src2));
        self
    }

    /// Set `sub` instruction to subtract the value of source register 2 from source register 1 and
    /// put the difference in the destination register
    ///
    /// # Arguments
    /// * `dst` - destination register
    /// * `src1` - source register 1
    /// * `src2` - source register 2
    pub fn with_sub(mut self, dst: Reg, src1: Reg, src2: Reg) -> Self {
        self.sub = Some((dst, src1, src2));
        self
    }

    /// Set `mul` instruction to multiply the values in the source registers and put the product in
    /// the destination register
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
