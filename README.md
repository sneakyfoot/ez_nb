notebook
=====

`notebook` is a lightweight command line helper for managing a rolling Markdown
notebook with daily, monthly, yearly, and someday pages. It can open notes in
your preferred editor, append entries from the terminal, list unfinished tasks,
search using `rg`, and keep the notebook repository in sync.

Installation
------------

You can install the CLI locally with [uv](https://github.com/astral-sh/uv):

```bash
uv tool install --from . notebook
```

After installation the `notebook` command will be available on your PATH. To update
the tool after making local changes, rerun the same command.

Usage
-----

```bash
notebook --help
```

Key commands:

- `notebook --daily` – open or append to today's note.
- `notebook --monthly --append` – pipe text directly into the monthly note.
- `notebook --search "keyword"` – search all Markdown notes using ripgrep.
- `notebook --tasks` – list incomplete checklist items across the notebook.
- `notebook --sync` – commit and push changes in the notebook git repository.

Configuration
-------------

- Set `EZ_NB_ROOT` to override the default notebook location (`~/notebook`).
- Use `--root PATH` for one-off runs pointing at a different notebook folder.
- Set `$EDITOR` or pass `--editor` to control which editor is used when opening
  notes (defaults to `nvim`).
