# project-sync

`project-sync` is a simple tool that uses `rsync` to provide file synchronization. It allows you to define multiple file synchronization jobs in a TOML configuration file and keeps remote or local destinations up to date with your source directories.

This tool is particularly useful for developers who need to sync projects between local machines and remote build servers or staging environments. It allows you to avoid editing remote files over SSH using tools like Emacs TRAMP or the VSCode Remote-SSH extension — instead, you can make edits locally and have them automatically synced to the remote destination.

## Features

* File change monitoring with automatic synchronization
* Hot-reloading of the configuration file upon edits
* Project filtering via command-line option
* Flexible multi-project configuration
* SSH-based remote sync (uses `rsync`)
* Configurable `rsync` options
* Automatic sync on startup
* Customizable ignore rules per sync job

## Installation

Pre-built binaries for major platforms are available on the [Releases](https://github.com/lyskov/project-sync/releases) page.

**Recommended**: Download the latest binary for your platform from the [Releases](https://github.com/lyskov/project-sync/releases) section and place it somewhere in your `PATH`, e.g.:

```bash
mv project-sync ~/.local/bin/
chmod +x ~/.local/bin/project-sync
```

### Building from Source

`project-sync` is written in Rust, so to build it from source you will need `cargo` installed. To build from source use:

```bash
cargo build --release
```

The resulting binary will be in `target/release/project-sync`.

Optionally, you can copy it somewhere in your `PATH`:

```bash
cp target/release/project-sync ~/.local/bin/
```

## Configuration

`project-sync` continuously watches for file system changes in the configured `source` directories. When a change is detected, synchronization is triggered automatically according to the specified job options.

Additionally, the utility watches its configuration file for edits. Any changes to the config file will be automatically reloaded, applying updated job definitions without restarting the application.

Create a `config.toml` file in `~/.config/project-sync/` or alongside the binary. Each sync job is defined using the `[[sync]]` table.

Global settings can be defined at the top level of the config file.

### Example Configuration

```toml
# Global options

debounce = 0.500 # seconds to debounce file change events

[[sync]]
name = 'example-local-sync'
source = '~/projects/my-local-dir/'
destinations = ['/Users/yourname/backup-dir/']

sync_on_start = true

ignore = '''
.git
*.tmp
node_modules
'''

[[sync]]
name = 'example-remote-sync'
source = '~/projects/my-project/'
destinations = ['username@remote-server:~/my-project']

sync_on_start = true
options = '-az -e ssh --delete'

ignore = '''
.git
__pycache__
target
*.log
'''
```

### Field Reference

#### Global Options

* `debounce`: Debounce interval in seconds for file watching. Prevents excessive syncing during rapid file changes.

#### Per-Sync Job Fields

* `name` (string): Unique name for the sync job.
* `source` (string): Source directory. `~` is expanded to home.
* `destinations` (array of strings): List of destination paths. Can be local or remote via SSH.
* `sync_on_start` (boolean):: If `true`, will sync automatically when the tool starts.
* `options` (string, optional): Optional `rsync` options to override the default (`-az -e ssh`).

  * **Note**: If `options` is not specified, it defaults to `-az -e ssh`.
  * Including `--delete` in the options will cause files in the destination that are not present in the source to be removed. **Use with caution!**
* `ignore` (string, optional): Multiline string of patterns (one per line) to exclude from sync.

  * **Note**: If `ignore` is not specified, it defaults to ignoring `.git`.

> **Important Note on Trailing Slashes**: Like `rsync`, `project-sync` inherits the same behavior regarding trailing slashes in the source path:
>
> * If the source ends with a `/`, only the *contents* of the directory are copied to the destination.
> * If the source does **not** end with a `/`, the *directory itself* and its contents are copied into the destination.
>
> For example:
>
> ```bash
> rsync -a ~/mydir/ user@host:~/backup/   # copies contents of mydir
> rsync -a ~/mydir  user@host:~/backup/   # copies 'mydir' folder and its contents
> ```
>
> `project-sync` uses the same logic when invoking `rsync`, so be explicit with your intent when defining paths in your configuration.

## Running

By default, upon starting, `project-sync` runs all configured sync jobs that have `sync_on_start = true`, and then continues watching for changes.

You can dynamically filter which sync jobs to run by using the `--filter` (optional) command-line option. This will limit execution to jobs whose `destination` field contains the given substring:

```bash
project-sync --filter "@remote-server" sync-config.toml
```

This example reads your config file and runs only those jobs syncing to destinations that include `@remote-server`.

To view help or available CLI options:

```bash
project-sync --help
```

## Notes

* This tool relies on `rsync` being available on both the local and remote systems.
* For remote destinations, SSH access must be set up (preferably with key-based authentication).

## License

This project is licensed under the MIT License.
