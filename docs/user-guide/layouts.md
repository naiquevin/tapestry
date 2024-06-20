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

When the layout is `one-file-all-queries`, it's expected that the
`output` field of all queries must be the same. Otherwise the manifest
fails to validate. To avoid duplication, a related setting
`query_output_file` is provided.

If layout = `one-file-all-queries`, it's recommended to set
`query_output_file` and omit the `output` field for individual
queries.

If layout = `one-file-one-query`, then you must not set
`query_output_file`. Whether or not to set the `output` field for
individual queries is up to you.
