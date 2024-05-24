# Overview

Tapestry is built to address the peculiar concerns that I've had about
using libraries such as yesql, aiosql and the likes. While I agree
with the philosophy behind such libs&mdash;that SQL code is better
written as SQL directly rather than building it through ORMs, query
builders or worse, by string interpolation or concatenation&mdash;I've
had some concerns about using the approach in practice.

To understand more about the problems and how tapestry addresses them,
please check the [Rationale](../rationale.md) page.

The general idea behind this tool is, instead of users writing raw SQL
queries, have them write Jinja templates from which SQL queries as
well as pgTAP tests can be generated.

Here is a high level overview of how you'd use tapestry in your
project:

1. Create a directory inside your project where the templates will be
   located. The [`tapestry init`](commands.md/#init) command does this
   for you.

2. Add some information in the `tapestry.toml` [manifest](manifest.md)
   file:
   1. Lists of query templates, queries and test templates along with
      the mappings between them
   2. Location of query templates and test templates (input files)
   3. Location of where the output files are to be created
   4. etc...

3. Run [`tapestry render`](commands.md/#render) command to generate
   the SQL files, both for queries as well as tests.

4. Use a lib such as yesql, aiosql etc. to load the queries rendered
   by the previous step into the application runtime.

5. Use `pg_prove` to run the pgTAP tests, preferably as part of
   CD/CI. You'd need to implement some kind of automation for
   this. The github repo also includes a docker image that may help
   with this.
