use std::io;

use clap::CommandFactory;
use clap_complete::{generate, Shell};

use crate::opt::Command;

pub fn run(shell: Shell) {
    generate(shell, &mut Command::command(), "sqlx", &mut io::stdout())
}
