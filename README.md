Tapestry
========

Tapestry is a framework for writing (postgres)SQL queries and
([pgTAP](https://pgtap.org/)) tests using
[Jinja](https://github.com/mitsuhiko/minijinja) templates.

Tapestry is written in Rust but it can be used with applications
written in any programming language. It's purely a command line tool
that renders Jinja templates into SQL files. How to load the resulting
SQL code into memory and use it at runtime is entirely up to the
application.

This approach of loading SQL from files is not new. There are existing
libraries such as [yesql](https://github.com/krisajenkins/yesql),
[hugsql](https://github.com/layerware/hugsql) (Clojure),
[aiosql](https://github.com/nackjicholson/aiosql) (Python) etc. that
provide excellent abstractions for it. In absence of such a lib for
the language of your choice, it shouldn't take more than a few lines
of code to implement a simple file loader. In Rust apps, I simply use
the `include_str!` macro.

One limitation is that `tapestry` can only be used with PostgreSQL,
because of the tight coupling with `pgTAP`.

You may find this tool useful if,

1. you prefer direct SQL queries over ORMs or query builders to
   interact with RDBMS from application code

2. you are not averse to the idea of having (reasonable amount of)
   business logic inside SQL queries

In fact, if you have had concerns about point 2 i.e. having business
logic in SQL queries, perhaps `tapestry` addresses some of those
concerns. Learn more about the
[rationale](https://naiquevin.github.io/tapestry/rationale/) behind
this tool.

Current status
--------------

Tapestry is a work in progress. But I am presently using it in my
personal project so it has been tested for the basic use cases. A
working [example](examples/chinook) is also included in the repo which
you may try out.

The first tag/version is yet to be created and released.

Installation
------------

You can install `tapestry` directly from github,

``` shell
cargo install --git https://github.com/naiquevin/tapestry.git
```

### Additional dependencies

There's an additional dependency on
[pg_format](https://github.com/darold/pgFormatter) for formatting the
generated SQL files. It's recommended, but not a hard requirement.

Read the docs
-------------

Detailed documentation about `tapestry` can be found
[here](https://naiquevin.github.io/tapestry/)

Notable sections:

- [Rationale](https://naiquevin.github.io/tapestry/rationale/)
- [Installation](https://naiquevin.github.io/tapestry/user-guide/install/)
- [Getting started](https://naiquevin.github.io/tapestry/user-guide/getting-started/)
- [Commands](https://naiquevin.github.io/tapestry/user-guide/commands/)
- [Docker/Podman based testing workflow](https://naiquevin.github.io/tapestry/user-guide/docker/)

LICENSE
-------

MIT (See [LICENSE](LICENSE)).





