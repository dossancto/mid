# mid features and command reference

This document explains every user-facing `mid` command, output format, and
interactive table control.

Version 0.1.3 adds OS secret-manager password storage, secure-remote password
updates, connection-string retrieval, and configuration editing through `$EDITOR`.

## Command overview

```text
mid --help
mid remote list
mid remote add [CONNECTION_STRING] --name <NAME> [--database-type <TYPE>] [--is-secure]
mid remote edit
mid remote retrieve <NAME>
mid remote password <NAME> <PASSWORD>
mid remote remove <NAME>
mid remote switch <NAME>
mid status
mid list [--table-name <TABLE_NAME>] [--output-format <FORMAT>]
mid query [QUERY] [--output-format <FORMAT>] [--id <ID>]
mid query last [--output-format <FORMAT>]
mid history list
mid history last
```

Use `mid <command> --help` to inspect the options supported by the installed
version.

## Quick start

Add a connection and give it a memorable name:

```sh
mid remote add 'postgres://user:password@localhost/app' --name local-app
```

Activate it and confirm the selection:

```sh
mid remote switch local-app
mid status
```

Run a query:

```sh
mid query 'SELECT * FROM users LIMIT 20'
```

## Remote commands

Connections are called "remotes." The active remote is used by `query` and
`list`.

### Add a remote

```sh
mid remote add [CONNECTION_STRING] --name <NAME> [--database-type <TYPE>] [--is-secure]
```

Examples:

```sh
mid remote add 'postgres://user:password@localhost/app' --name app
mid remote add 'mysql://user:password@localhost/app' --name app-mysql
```

`--name` is required. If the connection string is omitted, `mid` opens an
interactive form. The form supports PostgreSQL and MySQL, masks passwords, and
uses `Tab` or `↑`/`↓` to move between fields. Use `←`/`→` to switch database
type, `Ctrl+V` to paste into the active field, `Enter` to save, and `q` to
cancel.

You can instead request an editor template:

```sh
mid remote add --name app --database-type postgres
mid remote add --name app-mysql --database-type mysql
```

This opens `$EDITOR` with a complete connection-string template. Saving and
closing the editor adds the remote; leaving the content empty cancels. Adding a
remote does not activate it, so use `remote switch` afterward.

### Store a password securely

Use `--is-secure` (or `-s`) to save only the password in the operating system's
secret manager:

```sh
mid remote add 'postgres://user:pass%23word@localhost/app' --name app --is-secure
mid remote add 'mysql://user:password@localhost/app' --name app-mysql -s
```

The config stores `is_secure = true` and a URL such as
`postgres://user:{pass}@localhost/app`. The secret manager stores the decoded
password (`pass#word` in the first example), under service `mid` and the remote
name. Connections restore and URL-encode the password automatically. An
available, unlocked OS secret manager is required.

Passwords embedded in connection URLs must be percent-encoded: `#` becomes
`%23`, `@` becomes `%40`, and `%` becomes `%25`. Quote the complete URL when
passing it through the shell.

### Update a secure remote's password

```sh
mid remote password app 'new#password'
```

Pass the raw password, not a URL or a percent-encoded password. This updates the
OS secret-manager entry; it does not change the database server's password or
the config URL. Non-secure remotes are rejected.

### Retrieve a connection string

```sh
mid remote retrieve app
```

Prints the connection URL directly. For secure remotes, the URL includes the
password retrieved from the OS secret manager, properly percent-encoded.
This output contains credentials; avoid sharing it or writing it to logs.

### Edit remote configuration

```sh
mid remote edit
```

Opens the global configuration file in `$EDITOR`. Secure passwords stay in the
secret manager; use `remote password` to change them. Remote names identify
secret-manager entries, so renaming a secure remote in the file does not move
its saved secret.

### List remotes

```sh
mid remote list
```

Prints the names of all configured remotes.

### Switch the active remote

```sh
mid remote switch app
```

Subsequent query and table commands use this remote.

### Remove a remote

```sh
mid remote remove app-mysql
```

The active remote cannot be removed. Switch to another remote first.

### Show status

```sh
mid status
```

Prints the name of the active remote.

## Configuration

Remote configuration is stored beneath the operating system's configuration
directory. On systems that use XDG paths, the location is:

```text
$XDG_CONFIG_HOME/mid/.midconfig.toml
```

On a typical Linux installation, this is `~/.config/mid/.midconfig.toml`.

Without `--is-secure`, connection strings, including passwords, are stored as
plain text. Secure remotes store a `{pass}` placeholder instead; other connection
details remain visible in the file. Protect this file with appropriate filesystem
permissions and do not commit or share it. Passwords supplied as command-line
arguments may also remain in shell history.

If an earlier development build saved a full URL in the secret manager, use
`mid remote password <NAME> '<RAW_PASSWORD>'` to replace it with just the password.

## Query commands

### Run a query

The default output is the interactive table:

```sh
mid query 'SELECT id, email FROM users'
```

Select another output format with `--output-format`:

```sh
mid query 'SELECT id, email FROM users' --output-format json
mid query 'SELECT id, email FROM users' --output-format sql
```

### Rerun a query by history ID

```sh
mid query --id 42
mid query --id 42 --output-format json
```

The ID can be found with `mid history list`.

### Rerun a recent query

```sh
# Latest query
mid query last

# Replay with JSON output
mid query last --output-format json
```

`query last` uses the newest history entry for the active remote.

## Output formats

The `query` and `list` commands support these formats:

| Format | Description |
| --- | --- |
| `table` | Interactive terminal table and the default format. |
| `json` | Pretty-printed JSON array. |
| `sql` | Multi-row `INSERT` statement generated from the result set. |

SQL export supports PostgreSQL and MySQL. It expects a query with a recognizable
`FROM <table>` clause and is intended for straightforward table queries rather
than arbitrary joins or derived tables.

## Interactive table

The table UI is used by query results and interactive table listing.

| Key | Action |
| --- | --- |
| `j` / `k` | Select the next or previous row. |
| `h` / `l` | Select the previous or next column. |
| `Home` / `G` | Select the first or last visible row. |
| `Shift+g` | Open the go-to-line popup. |
| `f` | Filter rows using the selected column. |
| `s` | Cycle the selected column through ascending, descending, and original order. |
| `Enter` | Open a value, or select a table in table-list mode. |
| `y` | Copy the selected value. |
| `e` | Edit the query using `$EDITOR`. |
| `u` | Prepare an update for the selected value (experimental). |
| `v` | Enter or leave multi-cell selection mode. |
| `q` | Quit. |

### Go to line

Press `Shift+g`, enter a displayed line number, and press `Enter`. Use `Backspace` to
edit the input. `Esc` or `q` closes the popup without moving.

### Filter

Select a column, press `f`, enter text, and press `Enter`. Matching is
case-insensitive and applies only to the selected column. Submit an empty filter
to restore all rows. `Esc` cancels without changing the current table.

### Copy

Press `y` to copy the full selected value. A successful copy briefly highlights
that exact cell in green. Clipboard support depends on the desktop or terminal
environment.

### Open values

Press `Enter` in query-result mode to open the full selected value. Table
previews may be shortened for performance; opening and copying use the full
value.

### Edit a query

Press `e` to open the query in the editor configured by `$EDITOR`. Saving and
closing the editor reruns the edited query.

`$EDITOR` is required, and its executable must be available in `PATH`:

```sh
export EDITOR=nvim
mid query 'SELECT * FROM users'
```

You can also set it for one invocation:

```sh
EDITOR=nvim mid query 'SELECT * FROM users'
```

### Update a selected value

Press `u` to prepare an update for the selected cell. This feature is
experimental and currently depends on the expected identifier-column behavior.
Review the generated query carefully before executing it.

### Sort a column

Select a column and press `s`. Repeated presses cycle through ascending,
descending, and original result order. The active sort is shown with `↑` or `↓`
in the column header. Selecting another column starts a new ascending sort.

### Select and update multiple values

Press `v` to enter selection mode, navigate to cells, and press `Enter` to
toggle each cell. Selected cells use a distinct color. In selection mode:

- `Shift+Enter` opens the selected values, joining cells from the same row with
  ` | ` and separating rows with newlines.
- `u` prepares updates for all selected cells. Assignments from the same row are
  combined in one `SET` clause; different rows produce separate `UPDATE`
  statements.
- `v` leaves selection mode and clears the selection.

Multi-cell update remains experimental. Review every generated statement before
applying it.

## List command

Open an interactive list of database tables:

```sh
mid list
```

Select a table with `j`/`k` and `Enter` to open its rows.

Query a table directly:

```sh
mid list --table-name users
mid list -t users
```

Choose an output format:

```sh
mid list --table-name users --output-format json
mid list --table-name users --output-format sql
```

## History commands

Executed query attempts are recorded with an ID, timestamp, SQL text, remote
name, and success state. IDs are assigned by the history store.

### List history

```sh
mid history list
```

### Display the latest entry for the active remote

```sh
mid history last
```

History is currently stored under the operating system's temporary directory as:

```text
mid/.midhistory.toml
```

It should not be treated as permanent storage.

## Help commands

```sh
mid --help
mid remote --help
mid query --help
mid query last --help
mid list --help
mid history --help
```
