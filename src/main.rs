use log::{debug, error, info};

mod parser;
mod pem;

use pem::Machine;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_micros()
        .init();

    let program_filepath = std::env::args().nth(1).unwrap_or_else(|| {
        debug!("No program file specified, defaulting to `./example_program.txt`");
        "./example_program.txt".to_string()
    });
    let program = parser::read_program(&program_filepath);

    let startup_memory_filepath = std::env::args().nth(2).unwrap_or_else(|| {
        debug!("No startup memory file specified, defaulting to `./startup_memory.txt`");
        "./startup_memory.txt".to_string()
    });
    let mut machine = Machine::new(parser::read_startup_memory(&startup_memory_filepath));
    machine.allow_data_race(
        std::env::var("ALLOW_DATA_RACE")
            .map(|s| s == "true")
            .unwrap_or(false),
    );

    match machine.compute(&program) {
        Ok(value) => info!("Result: {}", value),
        Err(e) => error!("Error: {}", e),
    }
}
