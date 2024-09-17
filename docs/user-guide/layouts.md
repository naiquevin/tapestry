# Layouts

Tapestry lets you control the layout of the query files i.e. how the
generated SQL is organized in files. It supports two ways at present:

1. `one-file-one-query`: Each SQL query will be written to a separate file
2. `one-file-all-queries`: All SQL queries will be written to a single file

To configure this, you need to specify the [`query_output_layout`](manifest.md#query_output_layout) key
in the manifest. The default option if not specified is
`one-file-one-query`.

## Layout and `queries[].output` field

Users may specify [`output`](manifest.md/#output) field
for every query, which is the path where the generated SQL output will
be written. If `output` is not specified, it's value is derived from
the query id. This works well for the `one-file-one-query` layout.

Example:

```toml
[[queries]]
id = "artists_long_songs@genre*limit"
template = "artists_long_songs.sql.j2"
conds = [ "genre", "limit" ]
```

The derived value of output for the above will be
`artists_long_songs-genre-limit.sql`.

But when the layout is `one-file-all-queries`, it's expected that the
`output` field of all queries must be the same. Otherwise the manifest
fails to validate. To avoid duplication, a related setting
`query_output_file` is provided.

If layout = `one-file-all-queries`, it's recommended to set
`query_output_file` and omit the `output` field for individual
queries.

Example:

```toml
query_output_layout = "one-file-all-queries"
query_output_file = "queries.sql"
```

**Tip**: If layout = `one-file-one-query`, then you must not set
`query_output_file`. Whether or not to set the `output` field for
individual queries is up to you.

## Layout and query tagging

[Name tagging of queries](query-tags.md) is mandatory when layout is
`one-file-all-queries`.

**Why?** (If you are curious): Name tags make parsing individual
queries from a single SQL file much more straightforward. The
[`status`](commands.md#status) command relies on parsing of individual
queries from a single output file.
