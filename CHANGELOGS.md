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
- `--version` / `-V` without a configured `version()` now report an unexpected
  argument instead of rendering `unknown`. Bundled shorts trigger the
  built-ins (`-vh` shows help, `-vV` shows the version).
- `--flag=value` on a `SetTrue` flag is now rejected instead of silently
  ignoring the value.
- User-defined `help` / `version` arguments (colliding `id` / `short` /
  `long`) are no longer hijacked: `-h` / `--help` behave as ordinary
  arguments when the built-ins are not injected.
- `render_help` now honors `self.styles` (a plain theme yields no ANSI
  escapes), falls back to `unknown` for a missing version like
  `render_version`, aligns columns on visible width, and lists `required`,
  default, possible-values, `env`, and conflict metadata for options and
  positionals.
- Subcommand versus positional ambiguity documented: subcommands take
  precedence on the first bare token, while `--` forces positional parsing
  (for example `prog -- build` treats `build` as a positional value).

### Added

- `ArgMatches::try_get_one` and `ArgMatches::try_get_many`: typed getters
  that distinguish missing arguments (`Ok(None)`) from parse failures
  (`Err(XarpError::Parse)`). Existing `get_one` / `get_many` behavior is
  unchanged and documented as returning `None` in both cases.
- `Xarp::try_get_matches`: parses `std::env::args()` without exiting, as a
  non-terminating alternative to `get_matches` for library code.
- `Xarp::try_get_matches_with_env`: deterministic parsing with an explicit
  environment map instead of the process environment (test-friendly).
- `XarpError::is_help`, `is_version`, and `is_parse` helpers for branching
  without an exhaustive match.
- `Effects::ALL`, `Effects::all()`, `Effects::from_bits()`, and
  `Effects::from_bits_truncate()`; `Debug` now lists set flag names.
- `BitOr<Style>` implementations for `Style`, `Effects`, and `Color` so
  styles compose with `|` (right-hand colors win, effects union).
- Definition validation now rejects unknown `conflicts_with` targets, empty
  ids, reserved short flags (`-`, `=`, whitespace), malformed longs, invalid
  subcommand names, and required positionals following optional ones.

### Changed

- `Style::paint` takes `self` by value and `Styled` owns the style, so
  temporaries like `Style::new().bold().paint("hi")` can be bound.
- `try_get_one` / `try_get_many` failures carry the same `--help` guidance
  as other parse errors.
- A `--` delimiter following an option that expects a value now reports a
  missing value instead of being swallowed as the value.
- Documented the `NO_COLOR` rule (any presence, including empty, disables
  color) and the double-underline (SGR 21) terminal caveat.
- Added 26 unit tests plus doc tests covering conflicts, defaults, env
  precedence, duplicates, positionals, subcommands, help/version routing,
  themes, styles, and effects.
