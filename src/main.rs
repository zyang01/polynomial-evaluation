use std::collections::HashMap;

mod pem;
use log::{error, info};
use pem::{
    types::{Addr, Const, Reg, Value},
    Instruction, Machine,
};

fn main() {
    env_logger::init();

    let mut machine = init_machine();
    let _example_prog = example_program();
    let _register_data_race_prog = register_data_race_program();
    let _place_holder_prog = place_holder_program();

    match machine.compute(&_register_data_race_prog) {
        Ok(value) => info!("Result: {}", value.0),
        Err(e) => error!("Error: {}", e),
    }
}

fn init_machine() -> Machine {
    Machine::new(HashMap::from_iter(
        ('A'..='Z')
            .enumerate()
            .map(|(i, c)| (Addr(i as u32), Value(c.to_string()))),
    ))
}

fn place_holder_program() -> Vec<Instruction> {
    Vec::from([
        Instruction::new().with_str(Reg(0), Addr(0)),
        Instruction::new().with_sub(Reg(1), Reg(0), Reg(0)),
    ])
}

fn example_program() -> Vec<Instruction> {
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

fn register_data_race_program() -> Vec<Instruction> {
    Vec::from([
        Instruction::new().with_ldi(Reg(0), Const(1)),
        Instruction::new().with_add(Reg(1), Reg(0), Reg(0)),
        Instruction::new().with_ldi(Reg(1), Const(3)),
    ])
}
