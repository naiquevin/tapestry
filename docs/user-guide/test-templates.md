# Test templates

Just like [`query_templates`](query-templates.md), `test_templates`
are also Jinja template files. But while one query template could be
used to generate several queries, one test template can be used to
generate only one `pgTAP` test file.

However, many test templates can be associated with a single query. In
other words, if multiple `pgTAP` test suites are to be written for the
same query, that's possible.

The test syntax is SQL only but with some additional functions
installed by `pgTAP`. If you are not familiar with `pgTAP` you can go
through it's documentation. Important thing to note is that the Jinja
variable `{{ prepared_statement }}` is made available to every test
template, and at the time of rendering, it will expand to the actual
query.

Let's look at a templates from the [chinook
example](https://github.com/naiquevin/tapestry/tree/main/examples/chinook).

Refer to the test template
[`songs_formats-afa_test.sql.j2`](https://github.com/naiquevin/tapestry/blob/main/examples/chinook/templates/tests/songs_formats-afa_test.sql.j2). The
first few lines are:

```sql
PREPARE song_formats (varchar, varchar) AS
{{ prepared_statement }};
```

Here we're using the `prepared_statement` Jinja variable to create a
prepared statement for the user session. The name of the prepared
statement is `song_formats` and it takes two positional args, both of
type `varchar`.

Later in the same file, the prepared statement is executed as part of
a `pgTAP` test case,

```sql
SELECT results_eq(
    'EXECUTE song_formats(''Iron Maiden'', ''Protected AAC audio file'')',
    $$VALUES
      ...
      ...
    $$,
    'Verify return value'
);
```

Check the
[`songs_formats-afa_test.sql`](https://github.com/naiquevin/tapestry/blob/main/examples/chinook/output/tests/songs_formats-afa_test.sql)
output file to see how the actual test file looks like.

!!! note

    Note that the SQL query that `prepared_statement` Jinja var
    expands to will always have `posargs` based placeholders, even if the
    [`placeholder`](manifest.md/#placeholder) config in manifest file is
    set to `variables`. That's the reason why the Jinja var is named
    `prepared_statement`

## Function instead of PS

Sometimes it's tedious to test for result sets returned by the
query. In such cases, it helps to manipulate the result returned by
the query and compare a derived property. E.g. If a query results too
many rows, it's easier to compare the count than the actual values in
the rows.

One limitation of prepared statements and the `EXECUTE` syntax for
executing them is that it's not sub-query friendly i.e. it's not
possible to execute a prepared statement as part of another query.

The following is **NOT** valid SQL

```sql
SELECT
    count(*)
FROM (EXECUTE song_formats ('Iron Maiden', 'Protected AAC audio file'));
```

In such cases, we can define a SQL function using the same
`prepared_statement` Jinja variable.

An example of this can be found in the chinook example -
[`all_artists_long_songs_test.sql.j2`](https://github.com/naiquevin/tapestry/blob/main/examples/chinook/templates/tests/all_artists_long_songs_test.sql.j2)

```sql
CREATE OR REPLACE FUNCTION all_artists_long_songs ()
RETURNS SETOF record
AS $$
{{ prepared_statement }}
$$ LANGUAGE sql;

BEGIN;
SELECT
    plan (1);

-- start(noformat)
-- Run the tests.
SELECT is(count(*), 204::bigint) from all_artists_long_songs() AS (artist_id int, name text, duration interval);
-- Finish the tests and clean up.
-- end(noformat)

SELECT
    *
FROM
    finish ();
ROLLBACK;
```

## Test fixtures

When it comes to automated tests, It's a very common requirement to
setup some test data to be able to write test cases. `pgTAP` tests are
not any different. In case of `pgTAP` one needs to create test data in
the database.

Since `pgTAP` tests are just SQL files, test data creation can be done
using SQL itself in the same file. Reusable setup code can also be
extracted into SQL functions that can be created as part of importing
the database schema.

The chinook directory doesn't include an example of this. But here's
an example from one of my real projects that uses `tapestry`.

In my project, there are two entities `categories` and `items` (having
tables of the same names) with `one-to-many` relationship i.e. one
category can have multiple items.

In several `pgTAP` tests, a few categories and items need to be
created. To do this, a function is defined as follows,

```sql
CREATE OR REPLACE FUNCTION tapestry.setup_category_n_items (cat_id varchar, item_idx_start integer, item_idx_end integer)
    RETURNS void
    AS $$
    INSERT INTO categories (id, name)
        VALUES (cat_id, initcap(replace(cat_id, '-', ' ')));
    INSERT INTO items (id, name, category_id)
    SELECT
        'item-' || t AS id,
        'Item ' || t AS name,
        cat_id AS category_id
    FROM
        generate_series(item_idx_start, item_idx_end) t;
$$
LANGUAGE sql;
```

And then it's used in `pgTAP` tests like this, 

```sql
...

BEGIN;
SELECT plan(1);

-- Fixtures
-- create 2 categories, 'cat-a' and 'cat-b' each having 5 items
SELECT
    tapestry.setup_category_n_items ('cat-a', 1, 5);
SELECT
    tapestry.setup_category_n_items ('cat-b', 6, 10);

-- Test cases

...
```






