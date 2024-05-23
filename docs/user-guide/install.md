# Installation

Until tapestry is published to crates.io, you can install it directly
from github,

```shell
    cargo install --git https://github.com/naiquevin/tapestry.git
```

### Additional dependencies

Tapestry depends on [pg_format](https://github.com/darold/pgFormatter)
for formatting the generated SQL files. It's not a hard requirement
but recommended.

On MacOS, it can be installed using homebrew,

```shell
    brew install pgformatter
```

Note that you need to install `pg_format` on the machine where you'd
be rendering the SQL files using `tapestry` e.g. on your workstation
and/or the build server.

## Dependencies for running tests

If you are using tapestry to render tests (which you should, because
that's what the tool is meant for!) then you'd also need the `pgTAP`
extension and the `pg_prove` command line tool.

`pgTAP` can be easily built from source. Refer to the instructions
[here](https://pgxn.org/dist/pgtap/).

You can install `pg_prove` from a CPAN distribution as follows:

```shell
sudo cpan TAP::Parser::SourceHandler::pgTAP
```

Refer to the [pgTAP installation
guide](https://pgtap.org/documentation.html#installation) for more
details.

As `tapestry` is a postgres specific tool, it goes without saying that
you'd need a working installation of postgres to be able to run the
tests. Please refer to the [official
documentation](https://www.postgresql.org/download/) for that.
