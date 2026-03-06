# AI agent skills

!!! note

    Experimental feature

The tapestry git repository includes a agent skills file that you can
copy/import into your AI coding agent. It's located under the
`.agents` directory in the root of the git repo.

Example: If you are a claude code user, then copy the
`.agents/skills/naiquevin-tapestry` directory in this repo to your
`~/.claude/skills` directory. You may also clone the repo and symlink
to it (which is how I use it).

Thereafter you can either explicitly invoke the skill from claude
code. Claude (or any coding agent) should also be able to infer the
skill if you ask it to generate SQL from a project that contains the
tapestry manifest file `tapestry.toml` anywhere inside it.

## Refer

- [Skills in claude code](https://code.claude.com/docs/en/skills)
- [The agent skills open standard](https://agentskills.io/home)

