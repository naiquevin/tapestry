# Chinook example

This directory contains an example `tapestry` "project". It's based on
the publicly available [chinook
database](https://github.com/lerocha/chinook-database).

The rendered SQL files are also committed to git repo. Even in real
projects, you should do so.

For ease of testing, this dir also provides a `run-tapestry` shell
script that can be used to run `tapestry` without having to install it
(it simply executes `cargo run` with correct path to the Cargo
manifest file).

To actually run the tests, you will need a working installation of
postgresql with the `pgTAP` extension and `pg_prove` CLI tool
installed.

This repo also provides a [docker based workflow](../docker) that
takes care of installing the dependencies.

You will need to,

1. Build the docker image and start the container. This will start the
   postgresql server. Refer to the [README](../docker/README.md) file
   for this.

2. Create a directory on your host machine as follows,

```shell
mkdir ~/tapestry-data
cd ~/tapestry-data
mkdir schema
cd schema
wget https://github.com/lerocha/chinook-database/releases/download/v1.4.5/Chinook_PostgreSql_SerialPKs.sql
```

Now `cd` into the examples/chinook dir again and render the SQL
files. Then rsync the generated `pgTAP` test files to the temp
directory created above.

```shell
./run-tapestry render
rsync -r output/tests ~/tapestry-data/
```

Now start another container using the same image

```shell
podman run -it \
    --rm \
    --network podman \
    -v ~/tapestry-data/:/tmp/tapestry-data/ \
    --env PGPASSWORD=secret \
    --env PGHOST=$(podman container inspect -f '{{.NetworkSettings.IPAddress}}' taptestbed) \
    tapestry-testbed:latest \
    run-tests -d chinook_serial
```

Note that the chinook database schema file that we downloaded takes
care of creating the database with the name `chinook_serial`. Hence
we've omitted the `-c` flag for the `run-tests` command. Refer to the
[docker workflow's README](../../docker/README.md) for detailed
information about this.
