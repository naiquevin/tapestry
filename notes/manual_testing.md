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
