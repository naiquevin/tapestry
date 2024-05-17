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
