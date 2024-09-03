# Configuring pg_format

`tapestry` can be configured to use [`pg_format`]() for formatting the
rendered SQL files. This makes sure that,

* the rendered SQL files have consistent indentation
* you don't need to worry about SQL indentation when writing Jinja
  templates

It can be installed on MacOS as follows,

```bash
brew install pgformatter
```

During project initialization, if tapestry finds `pg_format`
installed on your system (and in `$PATH`), it will show it as one of
the formatter options. If you choose it, then following lines will be
added to the `tapestry.toml` manifest file.

```toml
[formatter.pgFormatter]
## (required) Location of the pg_format executable
exec_path = "pg_format"
## (optional) path to the pg_format conf file.
conf_path = "./.pg_format/config"
```

The behavior of `pg_format` tool in the context of `tapestry` can be
configured by adding a config file. The [sample config
file](https://github.com/darold/pgFormatter/blob/master/doc/pg_format.conf.sample)
in the `pg_format` github repo can be used for reference.

The [`tapestry init`](commands.md/#init) command also generates a
default config file, located at `.pg_format/config` (relative to the
manifest file) and looks like this,

```config
# Lines between markers 'start(noformat)' and 'end(noformat)' will not
# be formatted. If you want to customize the markers, you may do so by
# modifying this parameter.
placeholder=start\(noformat\).+end\(noformat\)

# Add a list of function to be formatted as PG internal
# functions. Paths relative to the 'tapestry.toml' file will also work
#extra-function=./.pg_format/functions.lst

# Add a list of keywords to be formatted as PG internal keywords.
# Paths relative to the 'tapestry.toml' file will also work
#extra-keyword=./.pg_format/keywords.lst

# -- DANGER ZONE --
#
# Please donot change the following config parameters. Tapestry may
# not work otherwise.
multiline=1
format=text
output=
```

As you can see, the generated file itself is well documented.

## Disallowed configuration

In the context of `tapestry`, some `pg_format` config params are
disallowed (or they need to configured only in a certain way) for
proper functioning of `tapestry`. These are explicitly defined with
the intended value in the config file and annotated with `DANGER ZONE`
warning in the comments. These must not be changed.

## Selectively opting out of SQL formatting

A commonly faced problem with formatting `pgTAP` tests using
`pg_format` is that hard coded expected values get formatted in a way
that could make the test case unreadable for humans.

Example: Consider the following `pgTAP` test case written in a test
template file,

```sql
SELECT results_eq(
    'EXECUTE artists_long_songs(''Rock'', 2)',
    $$VALUES
        (22, 'Led Zeppelin'::varchar, '00:26:52.329'::interval),
        (58, 'Deep Purple'::varchar, '00:19:56.094'::interval)
    $$,
    'Verify return value'
);
```

By default `pg_format` would format the above SQL snippet as follows,

```sql
SELECT
    results_eq ('EXECUTE artists_long_songs(''Rock'', 2)', $$
    VALUES (22, 'Led Zeppelin'::varchar, '00:26:52.329'::interval), (58, 'Deep Purple'::varchar, '00:19:56.094'::interval) $$, 'Verify return value');
```

To retain the readability, we need to preserve the user's custom
indentation. This is where the `placeholder` config param of
`pg_format` is useful

!!! note

    pg_format's `placeholder` config is not to be confused with [`placeholder`](manifest.md/#placeholder)
    config key in tapestry's manifest.

This can be done by adding `noformat` markers before and after the
snippet.

```sql
-- start(noformat)
SELECT results_eq(
    'EXECUTE artists_long_songs(''Rock'', 2)',
    $$VALUES
        (22, 'Led Zeppelin'::varchar, '00:26:52.329'::interval),
        (58, 'Deep Purple'::varchar, '00:19:56.094'::interval)
    $$,
    'Verify return value'
);
-- end(noformat)
```

If you want to customize the markers for whatever reason, you can
modify the `placeholder` param in the `pg_format` config file.
