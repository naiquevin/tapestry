# Query tags

## Name tagging queries

Typically, the output query files rendered by tapestry are intended to
be used by libraries such as `yesql`, `aiosql` etc. These libraries
require the queries to be "name-tagged". Tagging is done by simply
adding a comment before the query as follows,

```sql
-- name: my-query
-- A simple query
SELECT 1;
```

This way, these libraries can map the queries with the functions that
it generates in code. These functions wraps around the database
client/driver code and provides an easy interface for the user.

Name-tagging queries is specially makes sense when all queries are
rendered in the same output file (See `one-file-all-queries` in
[layouts](layouts.md)).

The following example is taken from [yesql's
README](https://github.com/krisajenkins/yesql):

```sql
-- name: users-by-country
SELECT *
FROM users
WHERE country_code = :country_code
```

...and then read that file to turn it into a regular Clojure function:

```clojure
(defqueries "some/where/users_by_country.sql"
   {:connection db-spec})

;;; A function with the name `users-by-country` has been created.
;;; Let's use it:
(users-by-country {:country_code "GB"})
;=> ({:name "Kris" :country_code "GB" ...} ...)
```

### Name tag format

The name tag is just a comment with a prefix `name: `. But if any
other comment lines are present before a query, then the name tag
should precede it.

```
<name tag>
<additional docstring if any>
<query>
```

<span style="color: green;">Correct &#9745;</span>

```sql
-- name: my-query
-- A simple query
SELECT 1;
```

<span style="color: red;">Incorrect &#9746;</span>

```sql
-- A simple query
-- name: my-query
SELECT 1;
```

### Deriving name tags from id

The name tagging config in the manifest file will look like this:

```toml
[name_tagger]
style = "kebab-case"
```

This will result in name tags to be added to query output files. By
default, the name tags are derived from the query
[ids](manifest.md/#id). The `style` setting allows us to control how
the id should be slugified to derive the name tag. For
e.g. `kebab-case` will cause all non-alphanumeric characters in the id
to be replaced by hyphens.

The other options for style are `snake_case` and `exact`.

### Custom name tags

The above method derives name tags from query ids. But yesql and
aiosql sometimes require the query names to be suffixed with specific
characters to indicate specific operations. Example: In yesql, the
name tags for INSERT/UPDATE/DELETE statements need to be suffixed with
`!`.

```sql
-- name: save-person!
UPDATE person
    SET name = :name
    WHERE id = :id
```

There are two ways to achieve this:

1. Specify `exact` as the `name_tagger.style`. Then the query
   [`id`](manifest.md/#id) itself to be used as the name tag (as it
   is).

2. Specify the optional [`queries[].name_tag`](manifest.md/#name_tag)
   field when defining the queries.

While it may seem like the first approach involves less effort, the
downside is that we'd be giving up on the [Naming
conventions](naming-conventions.md) that tapestry recommends.

Libraries such as yesql and aiosql usually don't allow special
characters in the name tags as they use them to generate functions in
code. So yesql recommends the name tags to be in `kebab-case` as
Clojure functions follow that convention, whereas aiosql needs the
name tags to be in `snake_case` as that's the requirement and also the
convention in Python.

### Disabling name tagging

Name tagging can be disabled by simply removing the `[name_tagger]`
TOML table from the manifest file.

Note however that name tagging cannot be disabled if the
[layout](layouts.md) is `one-file-all-queries`. Most tapestry commands
will fail with validation error in that case.

