# Manifest

Every `tapestry` "project" has a `tapestry.toml` file which is called
the _manifest_. It is in [TOML](https://toml.io/en/) format and serves
the dual purpose of configuration as well as a registry of the
following entities:

1. `query_templates`
2. `queries`
3. `test_templates`

The various sections or top level `TOML` keys are described in detail
below. When going through this doc, you may find it helpful to refer
to the [chinook
example](https://github.com/naiquevin/tapestry/tree/main/examples/chinook)
in the github repo. If you haven't checked the [Getting
started](getting-started.md) section, it's recommended to read it
first.

## placeholder

The `placeholder` key is for configuring the style of the placeholder
syntax for parameters i.e. the values values that are substituted into
the statement when it is executed.

Two options are supported:

### posargs

`posargs` is short for positional arguments. The placeholders refer to
the parameters by positions e.g. `$1`, `$2` etc. This is the same
syntax that's used for defining prepared statements or SQL functions
in postgres.

This option is suitable when your db driver or SQL library accepts
queries in prepared statements
syntax. E.g. [sqlx](https://github.com/launchbadge/sqlx) (Rust).

Default: The manifest file auto-generated upon running the [`tapestry
init`](commands.md/#init) command will have,

```toml
placeholder = posargs
```

### variables

When `placeholder=variables` placeholders are added in the rendered
query using the [variable substitution
syntax](https://www.postgresql.org/docs/current/app-psql.html#APP-PSQL-VARIABLES)
of postgres. The variable name in the query is preceded with colon
e.g. `:email`, `:department`

This option is suitable when your db driver or SQL library accepts
queries with
variables. E.g. [yesql](https://github.com/krisajenkins/yesql),
[hugsql](https://github.com/layerware/hugsql) (Clojure),
[aiosql](https://github.com/nackjicholson/aiosql) (Python)

Examples

=== "Template"

    ```sql
    SELECT
        *
    FROM
        employees
    WHERE
        email = {{ placeholder('email') }}
        AND department = {{ placeholder('department') }};
    ```

=== "placeholder = posargs"

    ```sql
    SELECT
        *
    FROM
        employees
    WHERE
        email = $1
        AND department = $2;
    ```

=== "placeholder = variables"

    ```sql
    SELECT
        *
    FROM
        employees
    WHERE
        email = :email
        AND department = :department;
    ```

!!! note

    Note that the `prepared_statement` Jinja variable available in
    [test templates](test-templates.md) will always have `posargs` based
    placeholders even if the `placeholder` config in manifest file is set
    to `variables`. That's the reason the Jinja var is named
    `prepared_statement`.

## query\_templates\_dir

Path where the query templates are located. The path is always
relative to the manifest file.

Default: The manifest file auto-generated upon running the [`tapestry
init`](commands.md/#init) command will have,

```toml
query_templates_dir = "templates/queries"
```

## test\_templates\_dir

Path where the query templates are located. The path is always
relative to the manifest file.

Default: The manifest file auto-generated upon running the [`tapestry
init`](commands.md/#init) command will have,

```toml
test_templates_dir = "templates/tests"
```

## queries\_output\_dir

Path to the output dir for the rendered queries. This path also needs
to be defined relative to the manifest file.

Default: The manifest file auto-generated upon running the [`tapestry
init`](commands.md/#init) command will have,

```toml
queries_output_dir = "output/queries"
```

A common use case to modify this config would be to store SQL files in
a directory outside of the tapestry "project" dir, so that only the
SQL files in that directory can be packaged into the build
artifact. There's no need to include the query/test template and the
`pgTAP` test files in the build artifact. E.g.

```toml
queries_output_dir = "../sql_queries"
```

## tests\_output\_dir

Path to the output dir for rendered `pgTAP` tests. The path is always
relative to the manifest file.

Default: The manifest file auto-generated upon running the [`tapestry
init`](commands.md/#init) command will have,

```toml
tests_output_dir = "output/tests"
```

## query\_output\_layout

[Layout](layouts.md) to be used for the generated query files. The two
options are:

1. `one-file-one-query`: Each SQL query will be written to a separate file

2. `one-file-all-queries`: All SQL queries will be written to a single file

It's optional. The default value is `one-file-one-query`.

Example:

```toml
query_output_layout = "one-file-all-queries"
```

## query\_output\_file

`query_output_file` is optional but it's use is valid only when the
[layout](#query_output_layout) is `one-file-all-queries`. It basically
saves the user from having to define the same [`output`](#output) for
all queries. Example:

```toml
query_output_layout = "one-file-all-queries"
query_output_file = "queries.sql"
```

Refer to the [Layouts](layouts.md) section of the user guide for more
info on this topic.

## formatter.pgFormatter

This section is for configuring the `pg_format` tool that `tapestry`
uses for formatting the rendered SQL files.

There two config params under this section:

### exec_path

Location of the `pg_format` executable.

### conf_path

Path to the `pg_format` config file. It can be used for configuring
the behavior of `pg_format` when it gets executed on rendered SQL. As
with all paths that we've seen so far, this one is also relative to
the manifest file.

Example

```toml
[formatter.pgFormatter]
exec_path = "pg_format"
conf_path = "./.pg_format/config"
```

As mentioned in the installation guide, `pg_format` is not a mandatory
requirement but it's recommended.

Upon running the [`tapestry init`](commands.md/#init) command, this
section will be included in the auto-generated manifest file only if
the executable `pg_format` is found on `PATH`. In that case, a default
`pg_format` config file will also be created.

To read more about configuring `pg_format` in the context of
`tapestry`, refer to the [pg_format](pg-format.md) section of the
docs.

## name\_tagger

`name_tagger` is a TOML table, which if present in the manifest will
cause the generated SQL queries to be [name
tagged](query-tags.md/#name-tagging-queries).

### style

`name_tagger.style` can be used to control how name tags will be
derived from query id. The two options are:

1. `kebab-case`
2. `snake_case`
3. `exact`

Any special characters in the query `id` will be replaced with an
appropriate character based on the above option &mdash; hyphen in case
of `kebab-case` and underscore in case of `snake_case`. The third
option `exact` is different in the sense that the query `id` will be
used as it is as the name tag.

Example:

```toml
[name_tagger]
style = "kebab-case"
```

!!! Note

    Note the autological naming of options `kebab-case` (with a hyphen)
    v/s `snake_case` (with an underscore).

## query\_templates

`query_templates` is an [array of
tables](https://toml.io/en/v1.0.0#array-of-tables) in `TOML`
parlance. So it needs to defined with double square brackets and can
be specified multiple times in the manifest file.

For every query template, there are two keys to be defined:

### path

It's where the Jinja template file is located relative to the
[`query_templates_dir`](#query_templates_dir) defined earlier in the
manifest. `path` itself is considered as the unique identifier for the
query template.

Use `.j2` extension as the convention for the query template file.

### all_conds

It's a set of values that will be converted to `cond__` Jinja
variables that can be referenced inside the template. Note that they
are defined in the manifest without the `cond__` suffix.

This field is optional. If not specified, an empty set is considered
as the default.

For documentation on how to write a `query_template`, refer to
[Writing query templates](query-templates.md)

Example: 

```toml
[[query_templates]]
path = "artists_long_songs.sql.j2"
all_conds = [ "genre", "limit" ]

[[query_templates]]
path = "songs_formats.sql.j2"
all_conds = [ "artist", "file_format", "album_name" ]
```

!!! Note

    When `all_conds` is not specified, it essentially means that the query
    is a valid SQL statement and not a Jinja template. Then why define it
    as a template? The answer to that is &mdash; so that it can be
    embedded in tests.

## queries

`queries` is an [array of
tables](https://toml.io/en/v1.0.0#array-of-tables) in `TOML`
parlance. So it needs to defined with double square brackets and can
be specified multiple times in the manifest file.

A query can be defined using the following keys,

### id

`id` is an identifier for the query.

### template

`template` is a reference to a [`query_template`](#query_templates)
defined previously in the manifest.

### conds

`conds` is a subset of the `all_conds` key that's defined for the
linked query template. It's an optional and if not specified, an empty
set will be considered by default.

### output

`output` is the path to the output file where the SQL query will be
rendered. It must be relative to the `queries_output_dir` config.

It's optional to specify the `output`. If not specified, the filename
of the output file will be derived by _slugifying_ the `id`. This
property allows us to use certain [Naming
conventions](naming-conventions.md) for giving suitable and consistent
names to the queries.

Example:

```toml
[[queries]]
id = "artists_long_songs@genre*limit"
template = "artists_long_songs.sql.j2"
conds = [ "genre", "limit" ]
```

The derived value of `output` for the above will be
`artists_long_songs-genre-limit.sql`.

### name_tag

`name_tag` can be optionally set to specify a custom name tag for the
query. Name tags are prefixed to the SQL queries as comments and they
are used by SQL loading libraries such as yesql, aiosql etc. Read more
about in [Name tagging queries](query-tags.md/#name-tagging-queries).

!!! Note

    A query will be tagged with the specified `name_tag` only if
    [`name_tagger`](#name_tagger) is set.

## test_templates

`test_templates` is an [array of
tables](https://toml.io/en/v1.0.0#array-of-tables) in `TOML`
parlance. So it needs to defined with double square brackets and can
be specified multiple times in the manifest file.

A `test_template` can be defined using the following keys,

### query

`query` is a reference to [`query`](#query) defined in the manifest.

### path

`path` is the path to the jinja template for the `pgTAP` test. It must
be relative to the `test_templates_dir`.

Use `.j2` extension as the convention for the test template file.

### output

`output` is the path where the `pgTAP` test file will be rendered. It
must be relative to the `tests_output_dir`. 

Specifying `output` for `test_templates` is optional. If not
specified, it will be derived from the file stem of `path` i.e. by
removing the `.j2` extension.

For detailed documentation on how to write a `test_template`, refer to
[Writing test templates](test-templates.md)
