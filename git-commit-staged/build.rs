use clap::CommandFactory;
use std::fs;
use std::path::Path;

#[path = "src/staged/cli.rs"]
mod cli_staged;

#[path = "src/working/cli.rs"]
mod cli_working;

fn main() {
    let out_dir = Path::new("man");
    fs::create_dir_all(out_dir).expect("failed to create man directory");

    // Generate man page for git-commit-staged
    let cmd = cli_staged::Args::command();
    let man = clap_mangen::Man::new(cmd);
    let mut buffer = Vec::new();
    man.render(&mut buffer).expect("failed to render man page");
    fs::write(out_dir.join("git-commit-staged.1"), buffer).expect("failed to write man page");

    // Generate man page for git-commit-working
    let cmd = cli_working::Args::command();
    let man = clap_mangen::Man::new(cmd);
    let mut buffer = Vec::new();
    man.render(&mut buffer).expect("failed to render man page");
    fs::write(out_dir.join("git-commit-working.1"), buffer).expect("failed to write man page");
}
