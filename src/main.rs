use std::collections::HashMap;

mod pem;
use pem::{
    types::{Addr, Const, Reg, Value},
    Instruction, Machine,
};

fn main() {
    let mut machine = init_machine();
    let instructions = program();
    println!("Instructions: {:#?}", instructions);

    match machine.compute(&instructions) {
        Ok(value) => println!("Result: {}", value.0),
        Err(e) => println!("Error: {}", e),
    }

    let _p2_instructions = program2();
    // println!("Instructions: {:#?}", p2_instructions);
}

fn init_machine() -> Machine {
    Machine::new(HashMap::from_iter(
        ('A'..='Z')
            .enumerate()
            .map(|(i, c)| (Addr(i as u32), Value(c.to_string()))),
    ))
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
