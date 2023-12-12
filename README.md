# Polynomial Evaluation Machine

## Summary

This is a Rust implementation of the **Polynomial Evaluation Machine** as outlined by the coding challenge. The repo contains a machine implementation and a parser for reading pre-initialized memory and program from plain text files.

| Filename                         | Description                                                                                                                                                                                                              |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `startup_memory.txt`             | Memory with addresses `0` to `25` initialized with values of the symbolic variables `A` to `Z`.                                                                                                                          |
| `example_program.txt`            | The example program provided in the challenge.                                                                                                                                                                           |
| `long_polynomial.txt`            | A program that _strongly_ evaluates to the example polynomial shown in the challenge.                                                                                                                                    |
| `pure_numeric_program.txt`       | A program that demonstrates `strong_eval()` can compute numeric expressions with no symbolic variables.                                                                                                                  |
| `change_order_program.txt`       | A program that shows `strong_eval()` could result in an equivalent expression with different operation order using the same operators, and does not resolve the resulting operations that only contain numeric operands. |
| `register_data_race_program.txt` | A program that causes a data race writing to register.                                                                                                                                                                   |

## Algorithm

### Polynomial Evaluation Machine (PEM)

The machine is created with pre-initialized memory, an array of registers, a program counter, and a priority queue for keeping track of in-flight operations.

At the beginning of a cycle, the machine converts all operations in the instruction into `InFlightOperation`s and adds them to the `pending_operations` priority queue. All memory and register reads are completed at this stage.

The machine then concludes the cycle by removing `InFlightOperation`s that are scheduled to complete, and performs all register/memory writes by operation outputs. Data races can be detected at this stage by identifying operation outputs that write to the same register/memory address.

### Expression Evaluation

There are two expression evaluation methods in this implementation - `weak_eval()` and `strong_eval()`.

`weak_eval()` simply evaluates the expression tree by generating the corresponding parentheses-enclosed String representation in the case of operations, or string conversion in the case of leaf nodes. It does so recursively until all nodes are evaluated.

`strong_eval()` takes into consideration the associative property (or lack thereof) for each operator, and only apply parentheses to ensure the resulting expression remains equivalent. It also resolves purely numeric operations where possible in the original order of operations, with overflows handled by wrap-around. The resulting expression could have a different order of operations due to applied associative properties, but should remain equivalent to the original expression.

## Design Choices

### Represent values as _Expression Tree Nodes_ rather than `String`s

This greatly improves time and space complexity since the machine is not evaluating expressions and manipulating strings for every operation. Only the program output's expression tree needs evaluating after execution completes. Only the numeric constants loaded by the program and pre-initialized symbolic variables are stored as concrete values in memory, and they are never cloned thanks to reference-counting pointers.

### Tracking `InFlightOperation`s with priority queue

This allows efficient extraction of completed operations from the pending queue.

### Data race detection

Read/Write race conditions are non-existent at the compiler level since reads and writes happen at different stages of the cycle. Race conditions for writes result in undefined behavior and therefore they are detected and errored by default. The user can allow programs with write race conditions to continue executing by setting the `ALLOW_DATA_RACE` environment variable to `true`. :see_no_evil:

Note that write race conditions are only possible for register writes with the given operation set. It is not possible to have more than one operation writing to the same memory address due to the fact that only one operation (`str`) performs memory writes and only one operation of each type can complete at any given cycle.

## Getting Started

To get started, ensure that Rust is installed and navigate to the repo's root directory.

The PEM can be run with a custom program (default: `example_program.txt`) and init memory config (default: `startup_memory.txt`):

```bash
cargo run example_program.txt startup_memory.txt

cargo run pure_numeric_program.txt
```

`debug` and `trace` log levels provide greater visibility on execution:

```bash
RUST_LOG=debug cargo run long_polynomial.txt

ALLOW_DATA_RACE=true RUST_LOG=trace cargo run register_data_race_program.txt
```

## Time Spent

The total time spent was 40+ hours. Please see attached the `git-log.txt` file for local commit history.
