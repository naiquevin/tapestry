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

## status

The `status` command can be used to preview the effect of running
[`tapestry render`](#render) command. It will list which output files
will be added, modified or remain unchanged if the `render` command is
run. This command will not actually write the output files.

Output of running `tapestry status` inside the
[examples/chinook](https://github.com/naiquevin/tapestry/tree/main/examples/chinook)
directory:

```shell
$ tapestry status
Query: unchanged: output/queries/artists_long_songs.sql
  Test: unchanged: output/tests/all_artists_long_songs_count_test.sql
Query: unchanged: output/queries/artists_long_songs-limit.sql
Query: unchanged: output/queries/artists_long_songs-genre-limit.sql
  Test: unchanged: output/tests/artists_long_songs-genre-limit_test.sql
Query: unchanged: output/queries/songs_formats-artist-album.sql
Query: unchanged: output/queries/songs_formats-artist-file_format-album.sql
  Test: unchanged: output/tests/songs_formats-afa_test.sql
```

In a way, it's sort of a _dry run_ for the `render` command.

A more effective use of this command though is with the
`--assert-no-changes` flag which will cause it to exit with non-zero
code if it finds any output files that would get added or modified
upon rendering. It's recommended to be run as part of CD/CI, to
prevent the user from mistakenly releasing code without rendering the
templates.

## summary

The `summary` command prints a tabular summary of all queries along
with their associated (query) templates and tests.
