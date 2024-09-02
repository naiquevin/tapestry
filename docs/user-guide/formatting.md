# Formatting SQL

If your SQL queries are even moderately complex, you'd want them to be
formatted, mainly for readability. But with tapestry, you don't write
the actual SQL by hand. Instead you write SQL code in bits and pieces
within Jinja2 templates. Trying to get the SQL formatted the way you
like using jinja2's whitespace control doesn't work well.

For that reason, tapestry takes care of formatting SQL at the time of
rendering. For that, it supports a bunch of popular SQL formatting
tools that the user may already have installed on their system. The
currently supported formatting tools are:

1. pg_format
2. sqlfluff
3. sqlformat (inbuilt)

The first two are external tools that tapestry "shells-out" to. Hence
they are expected to be installed on your system.

Tapestry also comes with it's own inbuilt formatter that can be used
in case none of the above tools are installed. It's powered by the
sqlformat-rs crate. You may choose this if you don't prefer to install
an additional system level dependency. Although the level of config
supported by sqlformat is quite rudimentary.

## Selecting a formatter

When you initialize a new tapestry project by running `tapestry init`,
it will try to find if any supported formatting tools are installed on
your system. Based on that, it will show a prompt for selecting the
tool of your choice.

```
$ tapestry init myproject
? Choose an SQL formatter
  None (no formatting)
> sqlformat (built-in)
  pg_format
  sqlfluff
[The above SQL formatters were found on your system and available for use. Choose one or None to opt out of formatting]
```

The option selected by default is `sqlformat` which is the
aforementioned inbuilt formatter.

`None` is also an option in case you'd like to opt out of SQL
formatting. In that case, tapestry will skip the formatting step
altogether.

## Configuring the formatter

- [sqlformat](sqlformat-rs.md)

- [pg_format](pg-format.md)

- [sqlfluff](sqlfluff.md)

## Support for more formatters

The underlying formatting component of tapestry is designed to be
extensible, so that support for more tools can be added without much
effort. If you want a particular SQL formatting tool to be supported,
feel free to open an issue or a PR on github.

