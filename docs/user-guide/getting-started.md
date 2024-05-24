# Getting started

This tutorial is to help you get started with tapestry. It's assumed
that the following software is installed on your system:

- [`tapestry`](install.md)
- [`pg_format`](install.md/#additional-dependencies)
- a working installation of PostgreSQL ([official
  docs](https://www.postgresql.org/download/))
- [`pgTAP` and `pg_prove`](http://127.0.0.1:8000/user-guide/install/#dependencies-for-running-tests)

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
us. This is because it found the `pg_format` executable on
`PATH`. Refer to the [`pg_format`](todo) section for more details.

## Adding a query_template to generate queries

Now we'll define a query template. But before that, you might want to
get yourself familiar with the [chinook database's
schema](https://github.com/lerocha/chinook-database?tab=readme-ov-file#data-model).

Suppose we have an imaginary application built on top of the chinook
database in which the following queries need to be run,

1. list all artists with their longest songs

2. list top 10 artists having longest songs

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

2. `all_conds` is a set of values that will be converted to `cond__`
   Jinja variables. In this case it means there are two `cond__` Jinja
   templates supported by the template - `cond__genre` and
   `cond__limit`. Note that they are defined in the manifest without
   the `cond__` suffix.

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
   linked query template. In the context of this query, only the
   corresponding `cond__` Jinja variables will have the value `true`,
   and the rest of them will be `false`.

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

And you'll notice some files created in our directory.

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

Here is what the generated output files look like:

=== "artists\_long\_songs.sql"

    ```sql
    SELECT
        ar.artist_id,
        ar.name,
        max(milliseconds) * interval '1 ms' AS duration
    FROM
        track t
        INNER JOIN album al USING (album_id)
        INNER JOIN artist ar USING (artist_id)
    GROUP BY
        ar.artist_id
    ORDER BY
        -- Descending order because we want the top artists
        duration DESC;
    ```

=== "artists\_long\_songs-limit.sql"

    ```sql
    SELECT
        ar.artist_id,
        ar.name,
        max(milliseconds) * interval '1 ms' AS duration
    FROM
        track t
        INNER JOIN album al USING (album_id)
        INNER JOIN artist ar USING (artist_id)
    GROUP BY
        ar.artist_id
    ORDER BY
        -- Descending order because we want the top artists
        duration DESC
    LIMIT $1;
    ```

=== "artists\_long\_songs-genre-limit.sql"

    ```sql
    SELECT
        ar.artist_id,
        ar.name,
        max(milliseconds) * interval '1 ms' AS duration
    FROM
        track t
        INNER JOIN album al USING (album_id)
        INNER JOIN artist ar USING (artist_id)
        INNER JOIN genre g USING (genre_id)
    WHERE
        g.name = $1
    GROUP BY
        ar.artist_id
    ORDER BY
        -- Descending order because we want the top artists
        duration DESC
    LIMIT $2;
    ```

As you can see, the output SQL is formatted by `pg_format`.

## Adding a test_template

Now that we've defined and rendered queries, let's add
`test_template`. Again there are two changes required - an entry in
the manifest file and the Jinja template itself.

Add the following lines to the manifest file.

```toml
[[test_templates]]
query = "artists_long_songs@genre*limit"
path = "artists_long_songs-genre-limit_test.sql.j2"
```

Here we're referencing the query `artists_long_songs@genre*limit`
hence this test is meant for that query. The `path` key points to a
test template file that we need to create. So let's create the file
`templates/tests/artists_long_songs-genre-limit_test.sql.j2` with the
following contents:

```sql
PREPARE artists_long_songs(varchar, int) AS
{{ prepared_statement }};

BEGIN;
SELECT
    plan (1);

-- start(noformat)
-- Run the tests.
SELECT results_eq(
    'EXECUTE artists_long_songs(''Rock'', 10)',
    $$VALUES
        (22, 'Led Zeppelin'::varchar, '00:26:52.329'::interval),
        (58, 'Deep Purple'::varchar, '00:19:56.094'::interval),
        (59, 'Santana'::varchar, '00:17:50.027'::interval),
        (136, 'Terry Bozzio, Tony Levin & Steve Stevens'::varchar, '00:14:40.64'::interval),
        (140, 'The Doors'::varchar, '00:11:41.831'::interval),
        (90, 'Iron Maiden'::varchar, '00:11:18.008'::interval),
        (23, 'Frank Zappa & Captain Beefheart'::varchar, '00:11:17.694'::interval),
        (128, 'Rush'::varchar, '00:11:07.428'::interval),
        (76, 'Creedence Clearwater Revival'::varchar, '00:11:04.894'::interval),
        (92, 'Jamiroquai'::varchar, '00:10:16.829'::interval)
    $$,
    'Verify return value'
);
-- Finish the tests and clean up.
-- end(noformat)

SELECT
    *
FROM
    finish ();
ROLLBACK;
```

The test syntax is SQL only but with some additional functions
installed by `pgTAP`. If you are not familiar with `pgTAP` you can go
through it's documentation. But for this tutorial, it's sufficient to
understand that the `{{ prepared_statement }}` Jinja variable is made
available to this template, and when it's rendered it will expand to
the actual query.

Let's run the `render` command again.

```shell
tapestry render
```

And now you should see the pgTAP test file created at
`output/tests/artists_long_songs-genre-limit_test.sql`.

!!! note

    Here the file
    stem of the test template `path` itself was used as the output file
    name. But it's also possible to explicitly specify it in the manifest
    file (see [output](manifest.md#output_1) in `test_templates` docs).

This is how the rendered test file looks like,

```sql
PREPARE artists_long_songs (varchar, int) AS
SELECT
    ar.artist_id,
    ar.name,
    max(milliseconds) * interval '1 ms' AS duration
FROM
    track t
    INNER JOIN album al USING (album_id)
    INNER JOIN artist ar USING (artist_id)
    INNER JOIN genre g USING (genre_id)
WHERE
    g.name = $1
GROUP BY
    ar.artist_id
ORDER BY
    -- Descending order because we want the top artists
    duration DESC
LIMIT $2;

BEGIN;
SELECT
    plan (1);
-- start(noformat)
-- Run the tests.
SELECT results_eq(
    'EXECUTE artists_long_songs(''Rock'', 10)',
    $$VALUES
        (22, 'Led Zeppelin'::varchar, '00:26:52.329'::interval),
        (58, 'Deep Purple'::varchar, '00:19:56.094'::interval),
        (59, 'Santana'::varchar, '00:17:50.027'::interval),
        (136, 'Terry Bozzio, Tony Levin & Steve Stevens'::varchar, '00:14:40.64'::interval),
        (140, 'The Doors'::varchar, '00:11:41.831'::interval),
        (90, 'Iron Maiden'::varchar, '00:11:18.008'::interval),
        (23, 'Frank Zappa & Captain Beefheart'::varchar, '00:11:17.694'::interval),
        (128, 'Rush'::varchar, '00:11:07.428'::interval),
        (76, 'Creedence Clearwater Revival'::varchar, '00:11:04.894'::interval),
        (92, 'Jamiroquai'::varchar, '00:10:16.829'::interval)
    $$,
    'Verify return value'
);
-- Finish the tests and clean up.
-- end(noformat)
SELECT
    *
FROM
    finish ();
ROLLBACK;
```

## Run tests

Assuming that all the above mentioned prerequisites are installed, you
can run the tests as follows,

```shell
sudo -u postgres pg_prove -d chinook --verbose output/tests/*.sql
```

If all goes well, the tests should pass and you should see output similar to,

```shell
1..1
ok 1 - Verify return value
ok
All tests successful.
Files=1, Tests=1,  0 wallclock secs ( 0.03 usr  0.01 sys +  0.01 cusr  0.00 csys =  0.05 CPU)
Result: PASS
```

## That's all!

If you've reached this far, you should now have a basic understanding
of what `tapestry` is and how to use it. Next, it'd be a good idea to
understand the [manifest file](manifest.md) in more detail.

!!! note

    The chinook example discussed in this tutorial can also be found in
    the github repo under the `examples/chinook` directory (there are a
    few more tests included for reference).
