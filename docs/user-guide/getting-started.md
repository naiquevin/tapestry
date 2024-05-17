# Getting started

This tutorial is to help you get started with tapestry. It's assumed
that the following software is installed on your system:

- `tapestry`
- `pg_format`
- a working installation of PostgreSQL
- `pgTAP` and `pg_prove`

## Sample database

For this tutorial, we'll use the
[chinook](https://github.com/lerocha/chinook-database) sample
database. Download and import it as follows,

```shell
wget -P /tmp/ https://github.com/lerocha/chinook-database/releases/download/v1.4.5/Chinook_PostgreSql_SerialPKs.sql
createdb chinook
psql -d chinook -f /tmp/Chinook_PostgreSql_SerialPKs.sql
```

## Init

We'll start by running the `tapestry init` command, which will create
the directory structure and also write a bare minimum manifest file
for us. In a real project, you'd run this command from within the main
project directory, so that the files can be committed to the same
repo. But for this tutorial, you can run it from any suitable location
e.g. the home dir `~/`

```shell
cd ~/
tapestry init chinook
```

This will create a directory named `chinook` with following structure,

```shell
$ cd chinook
$ tree -a --charset=ascii .
.
|-- .pg_format
|   `-- config
|-- tapestry.toml
`-- templates
    |-- queries
    `-- tests
```

Let's look at the `tapestry.toml` manifest file that has been created
(I've stripped out some comments for conciseness)

```shell
$ cat tapestry.toml
placeholder = "posargs"

query_templates_dir = "templates/queries"
test_templates_dir = "templates/tests"

queries_output_dir = "output/queries"
tests_output_dir = "output/tests"

[formatter.pgFormatter]
exec_path = "pg_format"
conf_path = "./.pg_format/config"

# [[query_templates]]

# [[queries]]

# [[test_templates]]
```

`placeholder` defines the style of generated queries. Default is
`posargs` (positional arguments) which will generate queries with
`$1`, `$2` etc as the placeholders. These are suitable for defining
prepared statements.

Then there are four toml keys for defining directories,

1. `query_templates_dir` is where the query templates will be located

2. `test_templates_dir` is where the test templates will be located

3. `queries_output_dir` is where the SQL files for queries will be
   generated

4. `tests_output_dir` is where the SQL files for pgTAP tests will be
   generated.

All directory paths are relative to the manifest file.

You may have noticed that the `init` command created only the
templates dirs. `output` dirs will be created when `tapestry render`
is called for the first time.

The `init` command has also created a `pg_format` config file for
us. This is because it found the `pg_format` executable on `PATH`. For
more details about `pg_format` integration, refer `<TODO>`

## Adding a query_template to generate queries

Now we'll define a query template. But before that, you might want to
get yourself familiar with the [chinook database's
schema](https://github.com/lerocha/chinook-database?tab=readme-ov-file#data-model).

Suppose we have an imaginary application built on top of the chinook
database in which the following queries need to be run,

1. list all artists with their longest songs

2. list top 5 artists having longest songs

3. list top 5 artists having longest songs, and of a specific genre

As you can see, we'd need different queries for each of the 3
requirements, but all have a common logic of finding longest songs per
artist. Using Jinja syntax, we can write a query template that covers
all 3 cases as follows,

```sql
SELECT
    ar.artist_id,
    ar.name,
    max(milliseconds) * interval '1 ms' AS duration
FROM
    track t
    INNER JOIN album al USING (album_id)
    INNER JOIN artist ar USING (artist_id)
{% if cond__genre %}
    INNER JOIN genre g USING (genre_id)
  WHERE
  g.name = {{ placeholder('genre') }}
{% endif %}
GROUP BY
    ar.artist_id
ORDER BY
-- Descending order because we want the top artists
    duration DESC
{% if cond__limit %}
  LIMIT {{ placeholder('limit') }}
{% endif %}
;
```

We've used some custom Jinja variables for selectively including parts
of SQL in the query. These need to be prefixed with `cond__` and have
to be defined in the manifest file (we'll come to that a bit later).

We have also used the custom Jinja function `placeholder` which takes
one arg and expands to a placeholder in the actual query. This will be
clear once we render the queries.

Let's save the above query template to the file
`templates/queries/artists_long_songs.sql.j2`.

And now we'll proceed to defining the query_template and the queries
that it can generate in the manifest file. Edit the `tapestry.toml`
file by appending the following lines to it.

```toml
[[query_templates]]
path = "artists_long_songs.sql.j2"
all_conds = [ "genre", "limit" ]
```

To define a `query_template` we need to specify 2 keys:

1. `path` i.e. where the template file is located relative to the
   `query_templates_dir` defined earlier in the manifest. `path`
   itself is considered as the unique identifier for the query
   template.

2. `all_conds` is a set of "cond__" Jinja variables that are supported
   by the template. In this case, there are two of them - "genre" and
   "limit" (without the `cond__` suffix).

We can now define three different queries that map to the same
query_template

```toml
[[queries]]
id = "artists_long_songs"
template = "artists_long_songs.sql.j2"
conds = []

[[queries]]
id = "artists_long_songs*limit"
template = "artists_long_songs.sql.j2"
conds = [ "limit" ]

[[queries]]
id = "artists_long_songs@genre*limit"
template = "artists_long_songs.sql.j2"
conds = [ "genre", "limit" ]
```

To define a query, we need to specify 3 keys, 

1. `id` is an identifier for the query. Notice that we're following a
   naming convention by using special chars `@` and `*`. Read more
   about [Query naming conventions](todo).

2. `template` is reference to the query template that we defined
   earlier.

3. `conds` is a subset of the `all_conds` key that's defined for the
   linked query template.

We've defined three queries that use the same template. In the first
query, both the `conds` that the template supports i.e. "genre" and
"limit" are false. In the second query, "limit" is true but "genre" is
false. In the third query, both "genre" and "limit" are true. Queries
will be rendered based on these variables and the `{% if cond__.. %}`
expressions in the template.

Don't worry if all this doesn't make much sense at this point. Things
will be clear when we'll run `tapestry render` shortly.

## Rendering

Now let's run the `tapestry render` command.

```shell
tapestry render
```

And now let's see the contents of the directory again, 

```shell
$ tree -a --charset=ascii .
.
|-- .pg_format
|   `-- config
|-- output
|   |-- queries
|   |   |-- artists_long_songs-genre-limit.sql
|   |   |-- artists_long_songs-limit.sql
|   |   `-- artists_long_songs.sql
|   `-- tests
|-- tapestry.toml
`-- templates
    |-- queries
    |   `-- artists_long_songs.sql.j2
    `-- tests
```


## Adding a test_template

## Run tests
