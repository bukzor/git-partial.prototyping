use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "git-commit-staged")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Commit staged changes at specific paths only")]
#[command(
    long_about = "Unlike `git commit -- paths`, this commits from the index, not the working copy.\n\n\
                  Arguments after -- are passed through to git commit,\n\
                  enabling -m, --amend, --fixup, -C, GPG signing, hooks, etc.\n\n\
                  Examples:\n\
                  \x20 git commit-staged src/ -- -m \"Add feature\"\n\
                  \x20 git commit-staged src/ tests/ -- --amend\n\
                  \x20 git commit-staged . -- --fixup HEAD~1"
)]
pub struct Args {
    /// Paths to commit (only staged changes at these paths)
    #[arg(required = true)]
    pub paths: Vec<PathBuf>,

    /// Show what would be committed without committing
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Arguments to pass through to git commit
    #[arg(last = true)]
    #[allow(clippy::struct_field_names)]
    pub passthrough_args: Vec<String>,
}
