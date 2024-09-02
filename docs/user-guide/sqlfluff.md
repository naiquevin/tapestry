# Configuring sqlfluff

`sqlfluff` is a feature rich SQL formatter that's written in Python
and hence is an external dependency for tapestry.

`sqlfluff` recognizes a file named `./.sqlfluff` a standard
configuration file to load config from. Tapestry capitalizes on this
so that there's no need to invent a new config format.

The configuration options for `sqlfluff` are quite extensive and well
documented -
https://docs.sqlfluff.com/en/stable/configuration/index.html.

During project initialization, if you choose `sqlfluff` as the
preferred formatter, then following lines will be added to your
`tapestry.toml` manifest file.

```toml
[formatter.sqlfluff]
# (required) Location of the sqlfluff executable
exec_path = "sqlfluff"
```

Additionally, it will also create the `.sqlfluff` config file
alongside the manifest file.

```cfg
[sqlfluff]
dialect = postgres
```

You may refer to [sqlfluff
documentation](https://docs.sqlfluff.com/en/stable/configuration/index.html)
to configure formatting as per your preferences.

!!! note

    Unlike in [pgFormatter's config](pg-format.rs), the path to the
    sqlfluff config file doesn't need to be explicitly specified in the
    manifest file. Similar to normal functioning of `sqlfluff`, config
    will be implicitly loaded from a file named `./.sqlfluff` in the
    current directory. Since tapestry commands are run from the same dir
    that this file is created in, it just works.

