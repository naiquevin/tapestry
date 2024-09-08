# Publishing documentation

## Setup

### Create a virtualenv

```bash
mkdir ~/.virtualenvs
cd ~/.virtualenvs
python -m venv mkdocs

```

### Install dependencies

```bash
cd /path/to/tapestry/repo
. ~/.virtualenvs/bin/mkdocs/bin/activate
pip install -r docs-requirements.txt
```

### Verify

To verify that the setup works locally, run the following command from
the project root dir.

```bash
mkdocs serve
```

## Publishing on github pages

### Repo settings on Github

This is a one time setup that needs to be done.

- Open the `Settings` tab on github repo page
- Click on `Pages` in the left menu
- Under `Build and deployment`, select `Source` = "Deploy from a
  branch" and "gh-pages" as the branch name.

### Publish using mkdocs

```bash
mkdocs gh-deploy --force
```

This will trigger a `pages-build-deployment` workflow on github that
can be viewed and monitored from the `Actions` tab.
