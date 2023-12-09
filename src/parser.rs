use std::{collections::HashMap, fs::read_to_string};

use log::info;

use crate::pem::{
    types::{Addr, Const, Reg, Value},
    Instruction,
};

/// Read startup memory from file
///
/// # Arguments
/// * `filepath` - path to file containing startup memory
///
/// # Returns
/// * `HashMap<Addr, Value>` - startup memory
pub(crate) fn read_startup_memory(filepath: &str) -> HashMap<Addr, Value> {
    info!("Reading startup memory from `{filepath}`");

    let mut memory = HashMap::new();
    for (num, line) in read_to_string(filepath)
        .unwrap_or_else(|_| panic!("Unable to read startup memory from {filepath}"))
        .lines()
        .enumerate()
        .map(|(num, line)| (num + 1, line))
    {
        let mut split = line.splitn(2, ' ');
        let addr = split
            .next()
            .unwrap_or_else(|| panic!("No address on line {num}"));
        let value = split
            .next()
            .unwrap_or_else(|| panic!("No value on line {num}"));
        let addr = Addr(
            addr.parse::<u32>()
                .unwrap_or_else(|_| panic!("Invalid address on line {num}: {addr}")),
        );
        let value = Value(value.to_string());
        memory.insert(addr, value);
    }
    memory
}

/// Read program from file
///
/// # Arguments
/// * `filepath` - path to file containing program
///
/// # Returns
/// * `Vec<Instruction>` - program
///
/// # Panics
/// * If there is an invalid operation
/// * If there is an invalid operand
/// * If there is a missing semicolon at the end of the program
pub(crate) fn read_program(filepath: &str) -> Vec<Instruction> {
    info!("Reading program from `{filepath}`");

    let mut program = Vec::new();
    let mut curr_inst: Option<Instruction> = None;

    for (num, line) in read_to_string(filepath)
        .unwrap_or_else(|_| panic!("Unable to read program from {filepath}"))
        .lines()
        .enumerate()
        .map(|(num, line)| (num + 1, line))
    {
        curr_inst = curr_inst.or(Some(Instruction::new()));

        let mut split = line.split(' ');
        let op = split
            .next()
            .unwrap_or_else(|| panic!("No instruction on line {num}"));

        match op {
            "ldi" => {
                if let (Some(dst), Some(constant), None) =
                    (split.next(), split.next(), split.next())
                {
                    let dst = dst
                        .parse::<u32>()
                        .unwrap_or_else(|_| panic!("Invalid {op} register on line {num}: {dst}"));
                    let constant = constant.parse::<u32>().unwrap_or_else(|_| {
                        panic!("Invalid {op} constant on line {num}: {constant}")
                    });
                    curr_inst = curr_inst.map(|inst| inst.with_ldi(Reg(dst), Const(constant)));
                } else {
                    panic!("Invalid {op} operands on line {num}: `{line}`")
                }
            }
            "ldr" | "str" => {
                if let (Some(reg), Some(addr), None) = (split.next(), split.next(), split.next()) {
                    let reg = reg
                        .parse::<u32>()
                        .unwrap_or_else(|_| panic!("Invalid {op} register on line {num}: {reg}"));
                    let addr = addr
                        .parse::<u32>()
                        .unwrap_or_else(|_| panic!("Invalid {op} address on line {num}: {addr}"));

                    match op {
                        "ldr" => {
                            curr_inst = curr_inst.map(|inst| inst.with_ldr(Reg(reg), Addr(addr)))
                        }
                        "str" => {
                            curr_inst = curr_inst.map(|inst| inst.with_str(Reg(reg), Addr(addr)))
                        }
                        _ => unreachable!(),
                    }
                } else {
                    panic!("Invalid {op} operands on line {num}: `{line}`")
                }
            }
            "add" | "sub" | "mul" => {
                if let (Some(dst), Some(src1), Some(src2), None) =
                    (split.next(), split.next(), split.next(), split.next())
                {
                    let dst = dst
                        .parse::<u32>()
                        .unwrap_or_else(|_| panic!("Invalid {op} register on line {num}: {dst}"));
                    let src1 = src1
                        .parse::<u32>()
                        .unwrap_or_else(|_| panic!("Invalid {op} register on line {num}: {src1}"));
                    let src2 = src2
                        .parse::<u32>()
                        .unwrap_or_else(|_| panic!("Invalid {op} register on line {num}: {src2}"));

                    match op {
                        "add" => {
                            curr_inst =
                                curr_inst.map(|inst| inst.with_add(Reg(dst), Reg(src1), Reg(src2)))
                        }
                        "sub" => {
                            curr_inst =
                                curr_inst.map(|inst| inst.with_sub(Reg(dst), Reg(src1), Reg(src2)))
                        }
                        "mul" => {
                            curr_inst =
                                curr_inst.map(|inst| inst.with_mul(Reg(dst), Reg(src1), Reg(src2)))
                        }
                        _ => unreachable!(),
                    }
                } else {
                    panic!("Invalid {op} operands on line {num}: `{line}`")
                }
            }
            ";" => {
                program.push(curr_inst.unwrap());
                curr_inst = None
            }
            "#" => continue,
            _ => panic!("Invalid instruction on line {num}: {op}"),
        }
    }

    if curr_inst.is_some() {
        panic!("Missing semicolon at end of program")
    }

    program
}