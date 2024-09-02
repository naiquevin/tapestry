# Configuring sqlformat

`sqlformat` is the inbuilt formatter supported by tapestry. It's
implemented using the the
[sqlformat](https://crates.io/crates/sqlformat) crate.

It provides 3 basic config options:

1. `indentation`: Default is 4 spaces

2. `uppercase`: Whether or not reserved keywords should be converted to
   UPPERCASE.

3. `lines_between_queries`: No. of empty lines between two queries.

During project initialization, if you choose `sqlformat` as the
preferred formatter, then following lines will be added to your
`tapestry.toml` file.

```toml
[formatter.sqlformat-rs]
# (optional) No. of spaces to indent by
indent = 4
# (optional) Use ALL CAPS for reserved keywords
uppercase = true
# (optional) No. of line breaks after a query
lines_between_queries = 1
```
