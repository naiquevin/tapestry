# Manually testing tapestry

Notes about manually testing the tool against the
[chinook](../examples/chinook) example.

## Testing summary with '--all' option

```shell
cd examples/chinook
echo 'SELECT 1;' > output/queries/pqr-ost.sql
echo 'SELECT 1;' > output/tests/pqr-ost_test.sql
./run-tapestry summary --all
```

## Testing warning logs emitted during validation

To reproduce these cases, we need to,

1. Add a `query_templates` entry in the manifest file but not use it
   in any of the queries.
2. Create a query template file inside the `query_templates_dir` and
   not add it to the manifest
3. Create a test template file inside the `test_templates_dir` and not
   add it to the manifest file.

```shell
cat <<EOF >> tapestry.toml
[[query_templates]]
path = "unused_qt.sql.j2"
all_conds = []
EOF
echo 'SELECT 1;' > templates/queries/abc-foo.sql.j2
echo 'SELECT 1;' > templates/tests/pqr-ost_test.sql.j2'
```
