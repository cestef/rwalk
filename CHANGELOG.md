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

## v0.7.3 (2024-04-21)

## v0.7.2 (2024-04-21)

## v0.7.1 (2024-04-21)

## v0.7.0 (2024-04-21)

## v0.6.3 (2024-04-19)

## v0.6.2 (2024-04-17)

## v0.6.1 (2024-04-12)

## v0.6.0 (2024-03-23)

## v0.5.14 (2024-03-13)

## v0.5.13 (2024-03-13)

## v0.5.12 (2024-03-13)

## v0.5.11 (2024-03-12)

## v0.5.10 (2024-03-11)

## v0.5.9 (2024-03-09)

## v0.5.8 (2024-03-08)

## v0.5.7 (2024-03-08)

## v0.5.1 (2024-03-08)

## v0.5.0 (2024-02-27)

## v0.4.11 (2024-02-23)

## v0.4.10 (2024-02-23)

## v0.4.2 (2024-02-20)

## v0.4.1 (2024-02-19)

## v0.4.0 (2024-02-19)

## v0.3.20 (2024-02-16)

## v0.3.19 (2024-02-16)

## v0.3.18 (2024-02-15)

## v0.3.17 (2024-02-14)

## v0.3.16 (2024-02-14)

## v0.3.15 (2024-02-14)

## v0.3.14 (2024-02-14)

## v1.3.11 (2024-02-14)

## v0.3.9 (2024-02-14)

## v0.3.8 (2024-02-13)

## v0.3.7 (2024-02-13)

## v0.3.6 (2024-02-13)

## v0.3.5 (2024-02-13)

## v0.3.4 (2024-02-09)

## v0.3.3 (2024-02-09)

## v0.3.2 (2024-02-09)

## v0.3.1 (2024-02-09)

## v0.3.0 (2024-02-09)

## v0.2.18 (2024-02-05)

## v0.2.17 (2024-02-02)

## v0.2.16 (2024-02-02)

## v0.2.15 (2024-02-02)

## v0.2.14 (2023-12-22)

## v0.2.13 (2023-12-22)

## v0.2.11 (2023-11-24)

## v0.2.10 (2023-11-23)

## v0.2.9 (2023-11-23)

## v0.2.8 (2023-11-23)

## v0.2.7 (2023-11-21)
