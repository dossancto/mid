# Privacy and Data Storage

MID is a local database management application. MID does not intentionally collect, transmit, sell, or share users' database credentials, connection information, or SQL query history with the developer or third parties.

## Data Stored Locally

MID stores database connection configuration, database credentials, and SQL query history locally on the user's device.

MID uses the following local files:

- `.midconfig.toml` — stores database connection configurations, including complete database connection strings.
- `.midhistory.toml` — temporarily stores SQL query history.

Database passwords may also be stored separately using the operating system's credential store when secure credential storage is explicitly enabled by the user.

## Configuration Files

### Global Configuration

MID maintains a global `.midconfig.toml` file inside the operating system's user configuration directory.

The exact location depends on the operating system and the user's environment.

To display the paths currently used by MID, run:

```shell
mid info
```

Example output:

```text
Config file: /home/user/.config/mid/.midconfig.toml
History file: /tmp/mid/.midhistory.toml
```

### Project-Specific Configuration

MID can also use a project-specific configuration file located relative to the directory from which MID is being run:

```text
./mid/.midconfig.toml
```

When a project-specific configuration file exists, MID may use it instead of the global configuration file.

Because project-specific configuration is stored within the current working directory, users should take additional care when the directory is part of a source control repository, shared folder, synchronized directory, or backup.

Configuration files containing sensitive connection information should not be committed to public source control repositories.

## Database Connection Configuration

The `.midconfig.toml` file contains saved database connection configurations.

Connection strings are currently stored in plaintext and are not encrypted by MID.

Depending on the connection string provided by the user.

The configuration file persists between MID sessions.
Users should treat `.midconfig.toml` as potentially sensitive.

## Secure Credential Storage

MID provides optional secure storage for database passwords using the credential storage facilities provided by the operating system.

Secure credential storage is not automatically enabled for every database connection. It must be explicitly requested by the user when adding a remote connection.

For example, when using:

```shell
mid remote add
```

the `is_secure` option can be enabled to store the database password using the operating system's credential store rather than storing the password directly in the MID configuration file.

When secure storage is enabled:

- The database password is stored separately using the operating system's credential store.
- The password is not intended to be stored directly in `.midconfig.toml`.
- Other connection information remains stored in the MID configuration file.
- Access to the stored password is controlled through the user's operating system account and credential-store facilities.

A securely stored credential can be explicitly retrieved through MID using:

```shell
mid remote retrive "database"
```

where `"database"` identifies the corresponding saved database connection.

## SQL Query History

The `.midhistory.toml` file contains SQL query history entered or executed through MID.

The history file is stored in the operating system's temporary directory. Its lifetime may therefore depend on the operating system and its cleanup policies.

SQL queries may contain database names, table names, values, or other potentially sensitive or confidential information entered or queried by the user.

Users should avoid including credentials, access tokens, API keys, private keys, or other secrets directly in SQL queries whenever possible.

## Removing Stored Data

MID's locally stored data can be removed after closing the application.

To locate the global configuration and history files, run:

```shell
mid info
```

MID will display their locations:

```text
Config file: <path>
History file: <path>
```

To remove the global database configuration, delete the file shown as `Config file`.

To remove SQL query history, delete the file shown as `History file`.

If a project-specific configuration is being used, remove:

```text
./mid/.midconfig.toml
```

from the corresponding project or working directory.

The operating system may automatically remove the history file because it is stored in the system temporary directory.

Credentials stored using secure credential storage are maintained separately by the operating system's credential store. Deleting `.midconfig.toml` does not necessarily remove those credentials.

Securely stored credentials may need to be removed through MID, when supported, or through the credential-management facilities provided by the operating system.

Uninstalling MID may not automatically remove persistent configuration files or credentials stored in the operating system's credential store.

Securely Stored Credentials

When is_secure is enabled for a database connection, MID stores the password using the operating system's secure credential store. The credential is associated with the mid service and the configured database connection name.

The password is not stored in .midconfig.toml when secure credential storage is used.

Access to the stored password is controlled by the operating system's credential-management and access-control mechanisms. Applications cannot obtain the password simply by reading MID's configuration files or by accessing or deleting the MID executable.

Deleting MID does not automatically delete credentials stored by the operating system. Any remaining credentials continue to be protected by the operating system's credential store and are subject to its authentication, authorization, and access-control mechanisms.

If MID is no longer installed, users can view or remove any remaining credentials using the credential-management facilities provided by their operating system.

## Data Transmission

MID does not intentionally transmit stored database connection configurations, credentials, or SQL query history to the MID developer or third parties.

When a user connects to a database, MID communicates with the database server specified in the user's connection configuration. Data exchanged with that server is subject to the user's database configuration, network environment, and the policies of the database provider or administrator.

## User Responsibility

Users are responsible for securing their database credentials, database servers, operating system accounts, credential stores, project directories, and local MID application data.

Users should apply appropriate database security practices and use database accounts with only the permissions necessary for their intended use.

Project-specific configuration files should be handled carefully when projects are shared, synchronized, backed up, or stored in source control.

## Changes to This Disclosure

This disclosure may be updated if MID's storage, security, credential-management, or privacy behavior changes.

## Source Control Security

When using project-specific configuration, MID may create a `mid` directory containing database connection information.

It is **highly recommended** to exclude this directory from Git by adding it to the project's `.gitignore` or to your global Git ignore file:

```gitignore
mid/
```

Even when passwords are stored securely using the operating system's credential store, MID configuration files may still contain sensitive connection information.
