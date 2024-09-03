# Configuring sql-formatter

[sql-formatter](https://github.com/sql-formatter-org/sql-formatter) is
a Javascript library and command line tool for pretty printing SQL. It
supports multiple SQL dialects including postgresql, and hence makes
for a pretty good external tool that tapestry can use for formatting
the generated SQL.

You can easily install it using npm,

```shell
npm install -g sql-formatter
```

During project initialization, if `sql-formatter` is found installed
on your system (and in `$PATH`), it will be shown as one of the
formatter options. Upon choosing it, following lines will be added to
the `tapestry.toml` manifest file.

```toml
[formatter.sql-formatter]
# (required) Location of the sql-formatter executable
exec_path = "sql-formatter"
# (optional) path to the json conf file.
conf_path = "./.sql-formatter/config.json"
```

`sql-formatter` can be configured through a JSON file. The `init`
command also dumps a default JSON file at
`./.sql-formatter/config.json`, relative to the manifest file, with
the following contents:

```json
{
  "language": "postgresql",
  "tabWidth": 4,
  "keywordCase": "upper",
  "linesBetweenQueries": 2
}
```

Refer to the [sql-formatter
documentation](https://github.com/sql-formatter-org/sql-formatter?tab=readme-ov-file#configuration-options)
for more configuration options.
