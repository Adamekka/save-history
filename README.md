# save-history

`save-history` watches a directory and turns every real file change into a git commit.

It is meant for folders like game saves, notes, or other local state you want versioned automatically without thinking about it.

## What it does

- Watches a directory recursively for file changes
- Runs `git add -A` when something changes
- Creates a commit with the current local RFC 3339 timestamp as the commit message
- Optionally runs `git push` after each successful auto-commit with `--auto-push`
- Initializes the directory as a git repo if `.git/` does not exist yet

The watcher ignores `.git/`-internal filesystem events so git's own writes do not trigger extra empty commits.

## Installation

This project is not published to crates.io, so install it from source:

```bash
git clone https://github.com/adamekka/save-history.git
cd save-history
cargo build --release
```

The binary will be available at:

```bash
target/release/save-history
```

## Usage

```bash
save-history [OPTIONS] <DIRECTORY>
```

Examples:

```bash
# Watch a game save directory and auto-commit changes
save-history ~/Games/MyGame/Saves

# Also push each successful auto-commit to the current remote
save-history --auto-push ~/Games/MyGame/Saves
```

Current CLI options:

- `--auto-push` - Push each successful auto-commit to the current remote
- `-h`, `--help` - Print help
- `-V`, `--version` - Print version

## How it behaves

- If the target directory is not already a git repo, `save-history` runs `git init`
- After initialization it attempts an empty `init` commit
- Each detected change stages everything with `git add -A`
- Commits are only created when staged content actually differs from `HEAD`
- The directory is watched recursively, so nested files and folders are included

## Auto-push notes

`--auto-push` runs plain `git push` after a successful auto-commit.

That means:

- The repo should already have a configured remote and upstream branch
- Push failures are logged, but the watcher keeps running
- Auto-push does not change branch selection or remote configuration for you

## Operational notes

- Commits currently use the author `user <example@mail.com>`
- Commit messages are timestamps, not human-written summaries
- This tool is intentionally simple: one file change can produce one commit very quickly
- Because it uses `git add -A`, creations, modifications, deletions, and renames are all picked up

## Development

Run locally with Cargo:

```bash
cargo run -- [OPTIONS] <DIRECTORY>
```

Show CLI help:

```bash
cargo run -- --help
```
