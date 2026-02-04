use clap::CommandFactory;
use std::fs;
use std::path::Path;
use std::process::Command;

#[path = "src/staged/cli.rs"]
mod cli_staged;

#[path = "src/files/cli.rs"]
mod cli_files;

fn main() {
    // Embed git commit hash in version
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map_or_else(|| "unknown".to_string(), |s| s.trim().to_string());
    println!("cargo::rerun-if-changed=.git/HEAD");
    println!("cargo::rustc-env=GIT_HASH={git_hash}");
    let out_dir = Path::new("man");
    fs::create_dir_all(out_dir).expect("failed to create man directory");

    // Generate man page for git-commit-staged
    let cmd = cli_staged::Args::command();
    let man = clap_mangen::Man::new(cmd);
    let mut buffer = Vec::new();
    man.render(&mut buffer).expect("failed to render man page");
    fs::write(out_dir.join("git-commit-staged.1"), buffer).expect("failed to write man page");

    // Generate man page for git-commit-files
    let cmd = cli_files::Args::command();
    let man = clap_mangen::Man::new(cmd);
    let mut buffer = Vec::new();
    man.render(&mut buffer).expect("failed to render man page");
    fs::write(out_dir.join("git-commit-files.1"), buffer).expect("failed to write man page");
}
