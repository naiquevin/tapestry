# Changelog

## 0.2.0

Released on: 2024-09-08

- Implement a built-in SQL formatter using the `sqlformat-rs` crate
- Add support for two more external SQL formatters:
  [sqlfluff](https://sqlfluff.com/) and
  [sql-formatter](https://github.com/sql-formatter-org/sql-formatter).
- Update rust toolchain to version 1.80.1 in github workflows

## 0.1.1

Released on: 2024-09-07

- Functionality wise there's no change since `0.1.0`
- Upgraded rust toolchain for test jobs and added workflow for
  publishing release artifacts on github (for x86 linux and MacOS)
