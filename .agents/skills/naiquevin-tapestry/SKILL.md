---
name: naiquevin-tapestry
description: >
  Use this skill when the user wants to write a Jinja SQL query template
  for Tapestry, add a query_template or queries entry to tapestry.toml,
  create a pgTAP test template, or understand how cond__ variables and
  the placeholder() function work.
---

# Tapestry SQL Template Skill

Tapestry is a CLI tool that renders Jinja templates into PostgreSQL SQL
files. Users write `.sql.j2` template files, declare them in `tapestry.toml`,
then run `tapestry render` to produce plain `.sql` output files.

## When to use this skill
Use this skill when the user asks about SQL code generation and a
`tapestry.toml` file is found in the project directory.

## Key concepts

**Query templates** live in `templates/queries/` as `.sql.j2` files.
They use two Tapestry-specific Jinja constructs:

- `cond__<name>` — boolean variables that conditionally include SQL
  blocks using `{% if cond__genre %} ... {% endif %}`. Each condition
  must be declared in `all_conds` in the manifest (without the `cond__`
  prefix).
- `placeholder('arg_name')` — a Jinja function that expands to a
  positional argument (`$1`, `$2`, ...) in the rendered SQL. Use this
  wherever the application will pass a runtime value.

**Test templates** live in `templates/tests/` as `.sql.j2` files.
They use pgTAP syntax and have access to `{{ prepared_statement }}`,
which expands to the fully rendered query SQL.

## tapestry.toml manifest structure
```toml
[[query_templates]]
path = "my_query.sql.j2"         # relative to query_templates_dir
all_conds = [ "filter_a", "filter_b" ]

[[queries]]
id = "my_query"                   # no conditions active
template = "my_query.sql.j2"
conds = []

[[queries]]
id = "my_query*filter_a"          # naming convention: * separates filters
template = "my_query.sql.j2"
conds = [ "filter_a" ]

[[queries]]
id = "my_query@filter_b*filter_a" # @ separates entity scoping, * for limits
template = "my_query.sql.j2"
conds = [ "filter_a", "filter_b" ]

[[test_templates]]
query = "my_query@filter_b*filter_a"   # references a query id
path = "my_query_test.sql.j2"
```

## Naming conventions

- `*` in a query id separates optional filters (e.g. `*limit`, `*offset`)
- `@` scopes by an entity/dimension (e.g. `@genre`, `@country`)
- Rendered output filenames use `-` instead of `*` and `@`
  (e.g. `my_query-genre-limit.sql`)

## Workflow to follow when generating a template

1. Ask the user: what is the core SQL logic? What are the optional
   conditions (things that may or may not be included)?
2. In case it'd help to know the schema, ask the user where to look
   for schema. In many projects, schema is defined in `.sql`
   files. Ask the user for the exact location instead of guessing.
3. Write the `.sql.j2` template using `{% if cond__x %}` blocks and
   `{{ placeholder('x') }}` for runtime parameters.
4. Declare the `[[query_template]]` block with `all_conds`.
5. Declare one `[[queries]]` entry per meaningful combination of conds.
   Always include a base query with `conds = []`.
6. If the user wants tests, write a `[[test_templates]]` entry and a
   `.sql.j2` test file using pgTAP's `results_eq` or similar functions,
   referencing `{{ prepared_statement }}`.

## Example

Template (`templates/queries/artists_long_songs.sql.j2`):
```sql
SELECT ar.artist_id, ar.name, max(milliseconds) * interval '1 ms' AS duration
FROM track t
    INNER JOIN album al USING (album_id)
    INNER JOIN artist ar USING (artist_id)
{% if cond__genre %}
    INNER JOIN genre g USING (genre_id)
WHERE g.name = {{ placeholder('genre') }}
{% endif %}
GROUP BY ar.artist_id
ORDER BY duration DESC
{% if cond__limit %}
LIMIT {{ placeholder('limit') }}
{% endif %};
```

Manifest additions:
```toml
[[query_templates]]
path = "artists_long_songs.sql.j2"
all_conds = [ "genre", "limit" ]

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
