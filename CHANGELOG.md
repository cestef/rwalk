## v0.8.8 (2024-12-31)

### Feat

- add host validation in `--distributed`
- add `--distributed` to allow scanning multiple similar hosts at the same time

## v0.8.7 (2024-10-09)

### Feat

- add confirmation prompt before saving a configuration file that has interactive mode enabled + add `--yes` to skip prompts
- run lines one by one in interactive mode in case you paste multiple

### Fix

- opts table was displaying wordlist filter for transform
- zero chunk size error when filtered wordlist was very small
- contains,starts,ends filters were inverted
- filter empty args in interactive mode
- reported average in recursive mode when the recursion stops earlier than expected
- create parent dirs in interactive save

### Refactor

- move util files to utils

## v0.8.6 (2024-06-21)

### Fix

- implement ser + deser for keyvals to correctly parse the config
- create config file on `--open-config` if it doesn't exist

### Refactor

- change `--filter` multi-value delimiter to `;`
- use `=` instead of `:` for similar filter split

## v0.8.5 (2024-06-19)

### Feat

- better formatting for trees in scripting
- include git hash in the `--version`
- add `--capture` option to capture responses in the tree
- add `load` interactive command
- add `save` command in interactive mode to save the current config
- add aliases for interactive commands
- add scripting support for `--filter` and `--show`

### Fix

- make structs correctly available in interactive mode
- `--capture` wasn't able to capture the body of the responses
- remove comma separation for wordlists

### Refactor

- add a build script for git info + clap features

## v0.8.4 (2024-06-15)

### Feat

- add `--attributes` to override which attributes are crawled in spider mode
- allow comma separated values in vec args
- add `--external` to allow scanning external domains + improve tree display in spider mode
- add `--default-config` to display the default config in TOML format
- add `--open-config` to directly open the config in the default editor

## v0.8.3 (2024-06-10)

### Fix

- cz bump going crazy

## v0.8.2 (2024-06-10)

### Feat

- add --dir-script to override the directory detection method
- add --pretty option for JSON output
- add --ignore-scripts-errors (--ise)
- add a completions command to directly put the completions at the right place (--completions)
- switch to eyre for error handling

### Fix

- allow for access to the correct variables in directory scripting
- aborting throwing an error because of an empty channel
- print on root bar in recursive mode
- actually time out with max_time
- remove exclusive condition in directory detection
- apply formatting
- remove test panic

### Refactor

- remove banner
- create a Command trait for commands
- modularize interactive commands
- use oneshot channel for aborting
- --generate-completions is now obselete
- switch to eyre! instead of anyhow!

## v0.8.1 (2024-05-15)

### Feat

- add scripting to recursive and spider modes

### Fix

- do not consider a response as non-dir if html doesn't match

### Refactor

- do not re-init engine each iteration + remove logs when quiet
- avoid creating multiple scopes when running scripts

## v0.8.0 (2024-05-14)

### Feat

- Add scripting capabilities with rhai
- switch to quickjs for scripting
- Add javascript eval command to interact with the output data in -i mode
- add a remove command for interactive mode

### Fix

- remove useless mut

### Refactor

- rename data to tree in eval
- Switch to format! instead of "+" concatenation
- Switch to std::time::Instant instead of stopwatch

## v0.7.5 (2024-04-24)

### Fix

- Remove Cargo.lock from cz config
- Remove Cargo.lock

## v0.7.4 (2024-04-24)

### Feat

- switch to commitizen for linting commits
- add configuration files for commitlint and pre-commit

### Fix

- **ci**: Add Cargo.lock to committed files when bumping
- **cli**: Remove useless references to state
- add missing commitlint configuration file

### Refactor

- **ci**: Switch to actions/cache for caching actions
- **cli**: Switch to serde for dynamically setting values on the opts struct
- **spider**: Set default depth to DEFAULT_DEPTH + 1 to allow for proper scanning
