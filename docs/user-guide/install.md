# Installation

Currently, binaries for `x86_64` arch for Linux and MacOS can be
downloaded from the Github [release
page](https://github.com/naiquevin/tapestry/releases). Binaries for
`arm/aarch64` platform and Windows OS are not available yet.

If you have the rust tool chain installed on your machine, you can
build and install tapestry directly from github (without having to
clone the repo)

```shell
cargo install --git https://github.com/naiquevin/tapestry.git
```

### Additional dependencies

Tapestry can be configured to depend on external SQL formatting tools
for formatting the generated SQL files. In that case, it expects the
respective tool to be installed on the system.

Note that you need to install the formatting tools on the machine
where you'd be rendering the SQL files using `tapestry` e.g. on your
workstation and/or the build server.

But these are not hard requirements as tapestry also ships with a
basic inbuilt SQL formatter so that the generated files will be
properly formatted without the need of any external dependencies.

Refer to the [SQL formatting](formatting.md) section for more on this.

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
