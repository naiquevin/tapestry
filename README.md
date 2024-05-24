Tapestry
========

Tapestry is a framework for writing and maintaining (postgres)SQL
queries and ([pgTAP](https://pgtap.org/)) tests using Jinja templates.

Tapestry is written in Rust but it can be used with applications
written in any programming language. It's purely a command line tool
that renders jinja templates into SQL files. How to load the resulting
SQL code into memory and use it at runtime is entirely up to the
application.

This approach of loading SQL from files is not new, in fact there are
existing libraries such as
[yesql](https://github.com/krisajenkins/yesql),
[hugsql](https://github.com/layerware/hugsql) (Clojure),
[aiosql](https://github.com/nackjicholson/aiosql) (Python) etc. that
provide excellent abstractions for it. In absence of such a lib for
the language of your choice, it shouldn't take more than a few lines
of code to implement a simple file loader. In Rust apps, I simply use
the `include_str!` macro.

One limitation is that tapestry can be used with PostgreSQL only,
because of the tight coupling with `pgTAP`.

Installation
------------

Until tapestry is published to crates.io, you can install it directly
from github,

``` shell
    cargo install --git https://github.com/naiquevin/tapestry.git
```

### Additional dependencies

Tapestry depends on [pg_format](https://github.com/darold/pgFormatter)
for formatting the generated SQL files. It's a recommended but not a
hard requirement.

Read the docs
-------------

Detailed documentation about `tapestry` can be found
[here](https://naiquevin.github.io/tapestry/)

Notable sections:

- [Rationale](https://naiquevin.github.io/tapestry/rationale/)
- [Installation](https://naiquevin.github.io/tapestry/user-guide/install/)
- [Getting started](https://naiquevin.github.io/tapestry/user-guide/getting-started/)

LICENSE
-------

MIT (See [LICENSE](LICENSE)).





