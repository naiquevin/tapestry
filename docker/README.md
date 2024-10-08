# Container based workflow to run pgTAP tests

`tapestry` only generates SQL files for queries and `pgTAP` tests. To
be able to run the tests you need to install and setup:

1. PostgreSQL server
2. `pgTAP`, which is a postgres extension
3. `pg_prove`, which is a command line test runner/harness for `pgTAP`
   tests

While these can be setup manually, the `tapestry` github repo provides
a docker based workflow for easily running tests generated by
`tapestry` against a temporary pg database.

The relevant files can be found inside the `docker` directory under
project root.

!!! note

    I use podman instead of docker for managing containers. Hence all the
    docker commands in this doc have been actually tested using podman
    only. As podman claims CLI compatibility with docker, I am
    assuming that replacing `podman` with `docker` in the below
    mentioned commands should just work. If that's not the case,
    please create an issue on github.

## Build the docker image

```shell
cd docker
podman build -t tapestry-testbed -f ./Dockerfile
```

## Start container for postgres process

```shell
podman run --name taptestbed \
    --env POSTGRES_PASSWORD=secret \
    -d \
    -p 5432:5432 \
    tapestry-testbed:latest
```

Verify that the `5432` port is reachable from the host machine.

```shell
nc -vz localhost 5432
```

The above `podman run` command will create a container and start
it. After that you can manage the container using the `podman
container` commands

```shell
podman container stop taptestbed
podman container start taptestbed
```

## Running tests

The `pg_prove` executable is part of the image that we have built. But
to be able to run tests inside the container, we need to make the
database schema and the test SQL files accessible to it. For this we
bind mount a volume into the container when running it, using the
`--volume` option.

The container image has a bash script `run-tests` installed into it
which picks up the schema and the test SQL files from the mounted
dir.

The `run-tests` scripts makes certain assumptions about organization
of files inside the mounted dir. Inside the container, the dir must be
mounted at `/tmp/tapestry-data/` and there must be be two sub
directories under it:

1. `schema`: All SQL files inside this dir will be executed against
   the database server in lexicographical order to setup a temporary
   test database.

2. `tests`: All SQL files inside this dir will be considered as tests
   and specified as arguments to the `pg_prove` command.

Once such a local directory is created, you can run the tests as
follows,

```shell
podman run -it \
    --rm \
    --network podman \
    -v ~/tapestry-data/:/tmp/tapestry-data/ \
    --env PGPASSWORD=secret \
    --env PGHOST=$(podman container inspect -f '{{.NetworkSettings.IPAddress}}' taptestbed) \
    tapestry-testbed:latest \
    run-tests -c -d temptestdb
```

In the above command, `temptestdb` is the name of the db that will be
created by the `run-tests` script. If your schema files themselves
take care of creating the db, then you can specify that as the name
and omit the `-c` flag.

To know more about the usage of `run-tests` script, run,

```shell
podman run -it --rm tapestry-testbed:latest run-tests --help
```
