# Privacy and Data Storage

MID is a local database management application. MID does not intentionally collect, transmit, sell, or share users' database credentials, connection information, or SQL query history with the developer or third parties.

## Data Stored Locally

MID stores database connection configuration and SQL query history locally on the user's device.

MID uses two local files:

- `.midconfig.toml` — stores database connection configurations, including complete database connection strings.
- `.midhistory.toml` — temporarily stores SQL query history.

The exact locations of these files depend on the operating system and the user's environment.

To display the exact paths currently used by MID, run:

```shell
mid info
```

Example output:

```text
Config file: /home/user/.config/mid/.midconfig.toml
History file: /tmp/mid/.midhistory.toml
```

This command should be used when locating MID data for inspection, backup, or removal.

## Database Connection Configuration

The `.midconfig.toml` file contains saved database connection configurations.

Connection strings are currently stored in plaintext and are not encrypted by MID.

Depending on the connection string provided by the user, this file may contain sensitive information such as:

The configuration file persists between MID sessions.

## SQL Query History

The `.midhistory.toml` file contains SQL query history entered or executed through MID.

The history file is stored in the operating system's temporary directory. Its lifetime may therefore depend on the operating system and its cleanup policies.

SQL queries may contain database names, table names, values, or other potentially sensitive information entered or queried by the user.

## Removing Stored Data

MID's locally stored data can be removed after closing the application.

First, run:

```shell
mid info
```

MID will display the exact locations of the configuration and history files:

```text
Config file: <path>
History file: <path>
```

To remove saved database connections and connection strings, delete the file shown as `Config file`.

To remove SQL query history, delete the file shown as `History file`.

The operating system may automatically remove the history file because it is stored in the system temporary directory.

Uninstalling MID may not automatically remove the persistent configuration file, depending on the operating system and installation method.

## Data Transmission

MID does not intentionally transmit stored database connection strings or SQL query history to the MID developer or third parties.

When a user connects to a database, MID communicates with the database server specified in the user's connection configuration. Data exchanged with that server is subject to the user's database configuration, network environment, and the policies of the database provider or administrator.

## User Responsibility

Users are responsible for securing their database credentials, database servers, operating system accounts, and local MID application data.

Because database connection strings may contain plaintext credentials, users should use appropriate security practices and database accounts with only the permissions necessary for their intended use.

## Changes to This Disclosure

This disclosure may be updated if MID's storage, security, or privacy behavior changes.
