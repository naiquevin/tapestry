# Tapestry

Tapestry is a framework for writing and maintaining (postgres)SQL
queries and ([pgTAP](https://pgtap.org/)) tests using Jinja templates.

Tapestry is written in Rust but it can be used with applications
written in any programming language. It's purely a command line tool
that renders jinja templates into SQL files. An application can then
load the resulting SQL code into memory and use it at runtime.

This approach of loading SQL from files is not new, in fact there are
existing libraries such as
[yesql](https://github.com/krisajenkins/yesql),
[hugsql](https://github.com/layerware/hugsql) (Clojure),
[aiosql](https://github.com/nackjicholson/aiosql) (Python) etc. that
provide excellent abstractions for it. In absence of such a lib for
the language of your choice, it shouldn't take more than a few lines
of code to implement a simple file loader. In Rust apps, I simply use
the `include_str!` macro.

On the other hand, tapestry can be used with PostgreSQL only, because
of the tight coupling with pgTAP.

To understand the motivation behind this tool, please check the
[Rationale](rationale.md).
