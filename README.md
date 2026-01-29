notebook
=====

`nb` is a basic command line tool for managing a rolling Markdown notebook with daily, monthly, yearly, and someday pages.
Notes are stored in their corresponding subdirectory (except someday.md, which lives at the root), dated like yy-mm-dd.md.
When a note falls out of date, `nb` "rolls" the note to the next date. 

Rolling does this:
- Creates a new note with today's correct filename
- Constructs a new header for the correct date 
- Copies the remaining content to the new note.
- Strips all completed markdown tasks annotated by "- [x]"

Notes are synced via git. 
After running `nb init` to build the folder structure, do a standard git repo init with a remote origin.

Installation
------------
With nix profile:
`nix profile add github:sneakyfoot/ez_nb#nb`

Usage
-----

```bash
nb --help
nb edit --help
nb sync --help
```

Key commands:

- `nb`                                          - Defaults to `nb edit daily`.
- `nb edit`                                     - Defaults to `nb edit daily`.
- `nb edit [daily, monthly, yearly, someday]`   - Opens the corresponding note in $EDITOR.
- `nb sync`                                     - Syncs to and from remote git repository.

Configuration
-------------
- Currently defaults to `~/notebook` for notebook root.
- Uses $EDITOR for the edit command.
- If $EDITOR is unset, defaults to `nvim`
