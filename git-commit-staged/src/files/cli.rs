use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "git-commit-files")]
#[command(about = "Stage and commit files at specific paths")]
#[command(
    long_about = "Stages paths from working tree, then commits those staged changes.\n\n\
                  This combines `git add paths` + `git commit-staged paths` into one command.\n\n\
                  Arguments after -- are passed through to git commit,\n\
                  enabling -m, --amend, --fixup, -C, GPG signing, hooks, etc.\n\n\
                  Examples:\n\
                  \x20 git commit-files src/ -- -m \"Add feature\"\n\
                  \x20 git commit-files src/ tests/ -- --amend\n\
                  \x20 git commit-files . -- --fixup HEAD~1"
)]
pub struct Args {
    /// Paths to stage and commit
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
