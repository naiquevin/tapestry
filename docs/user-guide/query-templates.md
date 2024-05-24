# Writing query templates

Query templates are Jinja template files. One query template can be
used for generating multiple SQL queries. 

Often, an application needs to issue mostly similiar (or slightly
different) queries to the db based on user input. Some examples: 

- two queries that are exactly similar, except that one returns all
  columns i.e. `*` whereas the other returns only selected rows

- two queries that are exactly the same, except that one has a limit

- multiple similar queries but different combination of `WHERE`
  clauses

Using Jinja templates, it's possible to write a single query template
that can render multiple SQL queries. This is possible with a
combination of Jinja variables and `{% if .. %}...{% endif %}`
blocks. This is pretty much the main idea behind query templates.

Query templates need to be defined in the
[manifest](manifest.md/#query_templates) where we specify
[`all_conds`](manifest.md/#all_conds) which is a set of "cond" vars
that the template supports.

Let's look at a query template from the [chinook
example](https://github.com/naiquevin/tapestry/tree/main/examples/chinook)
distributed with the github repo.

```sql
SELECT
    track.name as title,
    artist.name as artist_name,
    {% if cond__album_name %}
      album.title as album_name,
    {% endif %}
    media_type.name as file_format
FROM
    album
    JOIN artist USING (artist_id)
    LEFT JOIN track USING (album_id)
    JOIN media_type USING (media_type_id)

{% if cond__artist or cond__file_format %}
  WHERE
  {% set num_conds = 0 %}
  {% if cond__artist %}
    artist.name = {{ placeholder('artist') }}
    {% set num_conds = num_conds + 1 %}
  {% endif %}

  {% if cond__file_format %}
    {% if num_conds > 0 %}
      AND
    {% endif %}
    media_type.name = {{ placeholder('file_format') }}
    {% set num_conds = num_conds + 1 %}
  {% endif %}
{% endif %}
;
```

The entry in the manifest file for the above `query_template` is,

```toml
[[query_templates]]
path = "songs_formats.sql.j2"
all_conds = [ "artist", "file_format", "album_name" ]
```

Because of the 3 `all_conds` defined in the manifest file, we have the
following Jinja variables available inside the Jinja template.

1. `cond__artist`
2. `cond__file_format`
3. `cond__album_name`

The `cond__artist` and `cond__file_format` vars are used for
conditionally including `WHERE` clauses. Because we want to add the
`WHERE` clause only if either of the two vars are true, and because we
want to add the `AND` operator only if both are true, nested `if`
blocks are used and a temp "counter" variable `num_conds` is defined
i.e. it's assigned to `0` and then incremented by 1 if the
`cond__artist` var is true.

The third variable `cond__album_name` is used for conditionally
including a column in the returned result.

## Query

Now let's look at how a `query` associated with this template is
defined in the manifest.

```toml
[[queries]]
id = "songs_formats@artist+album"
template = "songs_formats.sql.j2"
conds = [ "artist", "album_name" ]
output = "songs_formats__artist__album.sql"
```

In this query, only 2 of the 3 "cond" variables will be true.

As a total of 3 `all_conds` values are supported by the query
template, 8 different queries can be generated from it using different
subsets of `all_conds`.

```toml
[ ]
[ "artists" ]
[ "artists", "file_format" ]
[ "artists", "album_name" ]
[ "file_format" ]
[ "file_format", "album_name" ]
[ "album_name" ]
[ "artist", "file_format", "album_name" ]
```
