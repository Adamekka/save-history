use chrono::Local;
use clap::Parser;
use notify::{DebouncedEvent, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Parser)]
#[command(
    name = "SaveHistory",
    version = "0.0.0",
    about = "Watch a directory and save any changes using git."
)]
struct Cli {
    #[arg(value_name = "DIRECTORY")]
    path: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    if !cli.path.exists() {
        eprintln!("{} does not exist", cli.path.display());
        std::process::exit(1);
    }
    if !cli.path.is_dir() {
        eprintln!("{} is not a directory", cli.path.display());
        std::process::exit(1);
    }

    let git_dir = cli.path.join(".git");
    if !git_dir.exists() {
        match Command::new("git")
            .args(["init", cli.path.to_str().expect("valid UTF‑8 path")])
            .status()
        {
            Ok(status) if status.success() => {
                let _ = Command::new("git")
                    .args([
                        "-C",
                        cli.path.to_str().unwrap(),
                        "commit",
                        "--allow-empty",
                        "-m",
                        "init",
                        "--author",
                        "user <example@mail.com>",
                    ])
                    .status();
            }
            Ok(status) => {
                eprintln!("git init failed with status {}", status);
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("Failed to run git init: {}", e);
                std::process::exit(1);
            }
        }
    }

    let (tx, rx) = channel();
    let mut watcher = match notify::watcher(tx, Duration::from_secs(1)) {
        Ok(watcher) => watcher,
        Err(e) => {
            eprintln!("Failed to create filesystem watcher: {}", e);
            std::process::exit(1);
        }
    };
    if let Err(e) = watcher.watch(&cli.path, RecursiveMode::Recursive) {
        eprintln!("Failed to watch {}: {}", cli.path.display(), e);
        std::process::exit(1);
    }

    println!("Watching {}", cli.path.display());

    loop {
        match rx.recv() {
            Ok(event) => {
                if let DebouncedEvent::Error(ref err, _) = event {
                    eprintln!("watch error: {}", err);
                    continue;
                }

                let add_status = Command::new("git")
                    .args(["-C", cli.path.to_str().unwrap(), "add", "-A"])
                    .status();
                match add_status {
                    Ok(status) if status.success() => (),
                    Ok(_) => {
                        eprintln!("git add -A failed");
                        continue;
                    }
                    Err(e) => {
                        eprintln!("Failed to run git add: {}", e);
                        continue;
                    }
                }

                let diff_status = Command::new("git")
                    .args([
                        "-C",
                        cli.path.to_str().unwrap(),
                        "diff",
                        "--cached",
                        "--quiet",
                    ])
                    .status();
                let has_changes = match diff_status {
                    Ok(status) => !status.success(),
                    Err(e) => {
                        eprintln!("Failed to run git diff: {}", e);
                        false
                    }
                };
                if !has_changes {
                    continue;
                }

                let timestamp = Local::now().to_rfc3339();
                let commit_status = Command::new("git")
                    .args([
                        "-C",
                        cli.path.to_str().unwrap(),
                        "commit",
                        "-m",
                        &timestamp,
                        "--author",
                        "user <example@mail.com>",
                    ])
                    .status();
                match commit_status {
                    Ok(status) if status.success() => {
                        println!("Committed changes at {}", timestamp);
                    }
                    Ok(_) => {
                        println!("git commit made no changes at {}", timestamp);
                    }
                    Err(e) => {
                        eprintln!("Failed to run git commit: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("watch channel error: {}", e);
                break;
            }
        }
    }
}
