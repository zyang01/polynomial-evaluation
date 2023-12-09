mod pem;

use std::collections::HashMap;

use pem::{Instruction, Machine};

use crate::pem::types::{Addr, Const, Reg};

fn main() {
    let mut machine = Machine::new(HashMap::new());
    let instructions = program();
    println!("Instructions: {:#?}", instructions);

    let result = machine.compute(&instructions);
    println!("Result: {:?}", result);

    let p2_instructions = program2();
    println!("Instructions: {:#?}", p2_instructions);
}

fn program() -> Vec<Instruction> {
    Vec::from([
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
    ])
}

fn program2() -> Vec<Instruction> {
    Vec::from([Instruction::new()
        .with_sub(Reg(0), Reg(0), Reg(0))
        .with_str(Reg(0), Addr(0))])
}
