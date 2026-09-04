# Changelogs

All notable changes to `xarp` are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Conflicts no longer fire on `default_value`s. Conflict detection now uses
  explicitly selected arguments only (CLI or environment). Defaults never
  count as explicit.
- Subcommand mode now applies parent environment fallbacks, default values,
  `possible_values` validation, and conflict checks before delegating to the
  subcommand. Parent `required` arguments are skipped when a subcommand is
  present, since the subcommand is an alternative command.
- `ArgAction::Set` is now single-value last-wins. Repeated
  `--opt a --opt b` yields `b`. `ArgAction::Append` still collects all values.
- `SetTrue` with `default_value("true")` (or `"1"`, case-insensitive) now sets
  the flag. Falsy defaults leave it unset.
- Fixed `env` / `default_value` / `required` precedence. A falsy environment
  value (`"0"` / `"false"`) for `SetTrue` overrides any default and still
  enforces `required` instead of silently passing.
- Duplicate argument `id`s, `short`s, `long`s, and subcommand names now return
  a `XarpError::Parse` error instead of silently shadowing (first-wins).
  Built-in `help` / `version` injection skips on `id` / `short` / `long`
  collisions.
- Positional arguments now respect `ArgAction`: `Set` consumes one slot,
  `Append` (last positional only) collects all remaining values, `SetTrue`
  records presence as a flag. Non-last `Append` positionals are rejected.
- Short attached values now strip a leading `=`, so `-p=8080` behaves like
  `--port=8080`.
- Subcommand versus positional ambiguity documented: subcommands take
  precedence on the first bare token, while `--` forces positional parsing
  (for example `prog -- build` treats `build` as a positional value).

### Added

- `ArgMatches::try_get_one` and `ArgMatches::try_get_many`: typed getters
  that distinguish missing arguments (`Ok(None)`) from parse failures
  (`Err(XarpError::Parse)`). Existing `get_one` / `get_many` behavior is
  unchanged and documented as returning `None` in both cases.
