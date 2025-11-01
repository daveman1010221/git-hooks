use std::{env, fs, process};
use git_hooks::{normalize_commit_message, CommitError};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: commit-msg <commit-msg-file>");
        process::exit(1);
    }
    let path = &args[1];

    let raw = fs::read_to_string(path).unwrap_or_else(|_| {
        eprintln!("Failed to read commit message file");
        process::exit(1);
    });

    match normalize_commit_message(&raw) {
        Ok(cleaned) => {
            if cleaned != raw {
                if let Err(_) = fs::write(path, cleaned) {
                    eprintln!("Failed to rewrite cleaned commit message");
                    process::exit(1);
                }
                println!("✨ Commit message cleaned and validated.");
            } else {
                println!("✅ Commit message validated.");
            }
        }
        Err(CommitError::Empty) => {
            eprintln!("❌ Empty commit message");
            process::exit(1);
        }
        Err(CommitError::InvalidFormat) => {
            eprintln!("❌ Commit message must follow Conventional Commits format.");
            eprintln!("Example: `feat(parser): add support for nested rules`");
            process::exit(1);
        }
        Err(CommitError::Io) => {
            eprintln!("❌ I/O error");
            process::exit(1);
        }
    }
}
