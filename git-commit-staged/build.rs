use clap::CommandFactory;
use std::fs;
use std::path::Path;

#[path = "src/cli.rs"]
mod cli;

fn main() {
    let out_dir = Path::new("man");
    fs::create_dir_all(out_dir).expect("failed to create man directory");

    let cmd = cli::Args::command();
    let man = clap_mangen::Man::new(cmd);

    let mut buffer = Vec::new();
    man.render(&mut buffer).expect("failed to render man page");

    fs::write(out_dir.join("git-commit-staged.1"), buffer).expect("failed to write man page");
}
