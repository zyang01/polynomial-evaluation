mod pem;

use pem::Instruction;

fn main() {
    let instructions = program();
    println!("Instructions: {:#?}", instructions);

    let p2_instructions = program2();
    println!("Instructions: {:#?}", p2_instructions);
}

fn program() -> Vec<Instruction> {
    vec![
        Instruction::new().with_ldi(0, 1).with_ldr(1, 0),
        Instruction::new().with_ldi(2, 2).with_ldr(3, 1),
        Instruction::new(),
        Instruction::new(),
        Instruction::new(),
        Instruction::new().with_add(0, 0, 1),
        Instruction::new().with_add(2, 2, 3),
        Instruction::new(),
        Instruction::new().with_mul(0, 0, 2),
    ]
}

fn program2() -> Vec<Instruction> {
    vec![Instruction::new().with_sub(0, 0, 0).with_str(0, 0)]
}
