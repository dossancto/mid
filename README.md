# mid

`mid` is a terminal database client written in Rust for managing connections,
running queries, and quickly exploring results.

> `mid` is currently under active development. 

Version 0.1.3 adds secure password storage and remote credential management.

## Features

- PostgreSQL and MySQL connections.
- Interactive and `$EDITOR`-based connection setup.
- Optional OS secret-manager password storage with `--is-secure`.
- Secure password updates, connection-string retrieval, and config editing.
- TUI-based interactive query-result table.
- Column filtering and ascending/descending sorting.
- Multi-cell selection and generated multi-row updates.
- Query editing through `$EDITOR`.
- Per-remote query history and replay.

All commands, options, examples, TUI controls, and detailed feature explanations
are documented in [FEATURES.md](FEATURES.md).

## Status

| Capability | PostgreSQL | MySQL | SQLite |
| --- | --- | --- | --- |
| Connect and run queries | Working | Working | Planned |
| Interactive table output | Working | Working | Planned |
| JSON output | Working | Working | Planned |
| SQL `INSERT` export | Working | Working | Planned |
| List and select tables | Working | Working | Planned |
| Sort and filter results | Working | Working | Planned |
| Multi-cell selection | Working | Working | Planned |
| Update selected values | Experimental | Experimental | Planned |

## Requirements

- Rust and Cargo.
- PostgreSQL or MySQL access credentials.
- `$EDITOR` for editor-based connection setup and query editing (optional for
  other workflows).
- An available, unlocked OS secret manager when using secure remotes.

## Secure connections (0.1.3)

```sh
mid remote add 'postgres://user:pass%23word@localhost/app' --name app --is-secure
mid remote switch app
mid remote password app 'new#password'
```

Only the decoded password is saved in the OS secret manager. The config URL
contains `{pass}`, and `mid` restores the password when connecting. Encode
reserved password characters in connection URLs (`#` as `%23`); pass a raw
password to `remote password`. That command updates only secure remotes and
does not change the password on the database server.

Use `mid remote edit` to open config in `$EDITOR`, or `mid remote retrieve app`
to print the complete connection URL, including its password. Treat that output
as sensitive. Without `--is-secure`, passwords remain in the config file;
command-line credentials may also appear in shell history.

## Installation

Build the project:

```sh
cargo build --release
```

Install `mid` into Cargo's binary directory:

```sh
cargo install --path .
```

If you're testing a local build, replace `mid` in the documentation examples with
`cargo run --`:

```sh
cargo run -- --help
```

## Shell completion

Generate and install dynamic Fish completions in Fish's user completion directory:

```sh
mid generator --shell fish > ~/.config/fish/completions/mid.fish
```

Probably can work with other shells as well.

try 
```sh 
mid generator --shell {bash,zsh,fish} > ~/.config/{bash,zsh,fish}/completions/mid.{bash,zsh,fish}
```
after add the completions directory to your shell's config:

```sh
echo 'source ~/.config/{bash,zsh,fish}/completions/mid.{bash,zsh,fish}'
```

## Roadmap

### Safe mutation workflow

A dedicated `mutate` command is planned but is **not implemented yet**. The
intended workflow is to run mutations inside a transaction, report the affected
row count, and require explicit confirmation before committing:

```sh
# Planned syntax — not currently available
mid mutate 'DELETE FROM sessions WHERE expires_at < NOW()'
```

The goal is to provide guardrails for `UPDATE`, `DELETE`, `TRUNCATE`, and other
potentially destructive operations.

Other planned work includes:

- SQLite support.
- Durable history storage with SQLite.
- Safer and more general selected-cell updates.
- Local/project-specific remotes.

For the full command reference, see [FEATURES.md](FEATURES.md).
