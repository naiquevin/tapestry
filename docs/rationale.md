# Rationale

## Problems with using raw SQL in application code

For many years, I've believed that,

1. it's a good idea to write raw SQL queries (safely) for interacting
   with an RDBMS from application code using libs such as yesql,
   aiosql etc.

2. it's ok to add reasonable amount of business logic in the SQL
   queries, rather than using SQL merely for data access.

Still, I had some concerns about using these ideas in practice,
specially in serious projects.

### Unit testing SQL queries

Typically, unit tests are written against application code. As more
and more business logic gets moved out of the application and into SQL
queries, the queries become longer and more complex. Whereas the
application code is reduced to just making db calls using the
driver/client library. At this point, it makes more sense to test the
queries than the application code.

Fortunately for PostgreSQL, we have the excellent PgTAP extension that
makes it easy to write unit tests for raw queries. Just like the raw
queries themselves, pgTap tests are typically defined in SQL
files. But since the query and the tests are in separate files, it's
possible that one modifies the SQL query, but forgets to update the
tests, and the tests could still pass!

How to ensure that the tests actually run the exact same query that's
being run by the application?

### Maintenance overhead of multiple, slightly differing queries

An application often needs to issue similar queries but returning
different set of columns or with different `WHERE` clauses based on
user input. In such cases, a unique query needs to be written and
maintained for every combination of the input parameters.  This could
result in multiple queries that differ only slightly. If some core
part of the query needs a change, one needs to remember to update
multiple SQL files.

Moreover, higher level abstractions (e.g. yesql etc.) usually cache
queries in memory, so they require the queries to be given a name or
an identifier. Since the queries differ only slightly, trying to give
them unique names can be tricky. 

## How tapestry solves it?

Tapestry was built to specifically address the above problems and
concerns. It does so by generating actual queries as well as pgTAP
test files from Jinja templates, instead of having the user write raw
SQL.

### Query templates

* You write query templates instead of raw queries
* Multiple queries can be mapped to the same query template. Mapping
  is defined in the `tapestry.toml` manifest file.
* User defined jinja variables can be used for conditionally adding or
  omitting parts of the query e.g. a `WHERE` condition or column to
  return. These jinja vars are also defined in the manifest file.
* Thus, it's easy to generate and maintain multiple queries that are
  similar enough to be defined using a single query template.

### Test templates

* pgTAP tests are also written as jinja templates
* Test templates are mapped to queries, again in the manifest
  file. One query can be mapped to multiple test templates.
* When tapestry renders the final test file from a test template, a
  special jinja variable `{{ prepared_statement }}` gets expanded to
  the actual query that the test template is mapped to.
* This way, the generated test SQL file is guaranteed to have the
  exact same query which is used by the application code.

### Naming conventions

Tapestry suggests some conventions for naming queries consistently but
they are not mandatory. More about query and test naming conventions
in the user guide.
