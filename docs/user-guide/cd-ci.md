# CD/CI integration

If you are using tapestry in a real project, it's a good idea to
integrate it with your CD/CI workflows for the following purposes:

## Ensuring that the generated SQL files are not stale

Because of how tapestry works, you'd typically commit the generated
output files to version control. It then becomes important to ensure
that the `tapestry render` command is executed every time before a
release.

To help with this, tapestry provides the `--assert-no-changes` option
for the [`status`](commands.md#status) command.

```shell
$ tapestry status --assert-no-changes
Query: modified: output/queries/artists_long_songs.sql
  Test: modified: output/tests/all_artists_long_songs_count_test.sql
Query: unchanged: output/queries/artists_long_songs-limit.sql
Query: unchanged: output/queries/artists_long_songs-genre-limit.sql
  Test: unchanged: output/tests/artists_long_songs-genre-limit_test.sql
Query: unchanged: output/queries/songs_formats-artist-album.sql
Query: unchanged: output/queries/songs_formats-artist-file_format-album.sql
  Test: unchanged: output/tests/songs_formats-afa_test.sql
$ echo $?
1
```

If a commit is pushed without re-rendering the output files, the
command will fail.

## Ensuring test coverage

The [`coverage`](commands.md#coverage) command supports an option
`--fail-under` that can be used to make the command return with
non-zero exit code if the code coverage score (percentage of queries
that have at least 1 test) is below the provided threshold. Example:

```shell
$ tapestry coverage --fail-under=90 > /dev/null
$ echo $?
1
```

If a commit adds new SQL templates and queries but skips the tests,
causing the test coverage to drop below the standard, then this
command will fail.

## Running pgTAP tests

How to run the generated pgTAP tests against a blank postgres database
largely depends on your CD/CI and database setup. So tapestry doesn't
aim to provide a direct command for this. But you may be able to use
the [docker/podman based workflow](docker.md) as a starting point and
adapt it to your CD/CI platform.


