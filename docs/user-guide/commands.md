# Commands

The `tapestry` CLI provides the following commands:

## init

The `init` command can be used for scaffolding a new `tapestry`
"project". It will create the directory structure and also write a
bare minimum manifest file for us. In a real project, you'd run this
command from within the main project directory, so that the files can
be committed to the same repo. Example:

Running the following command,

```shell
tapestry init myproj
```

.. will create the following directory structure

```shell
$ cd myproj
$ tree -a --charset=ascii .
.
|-- .pg_format
|   `-- config
|-- tapestry.toml
`-- templates
    |-- queries
    `-- tests
```

## validate

The `validate` command checks and ensures that the
[manifest](manifest.md) file is valid. Additionally it also verifies
that the paths referenced in the manifest actually exist and are
readable.

## render

The `render` command renders all the template files into SQL files.

## summary

The `summary` command prints a tabular summary of all queries along
with their associated (query) templates and tests.


