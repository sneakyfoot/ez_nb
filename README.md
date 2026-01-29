notebook
=====

`nb` is basic command line tool for managing a rolling Markdown notebook with daily, monthly, yearly, and someday pages. 
Notes are stored in their corisponding subdirectory (except someday.md, which lives at root), dated like yy-mm-dd.md. 
When a note falls out of date, `nb` "rolls" the note to the next date. 

Rolling does this:
- Creates a new note with todays correct filename
- Constructs a new header for the correct date 
- Coppies the remaining context to the new note.
- Strips all completed markdown tasks annotated by "- [x]"

Notes are synced via git. 
After running `nb init` to build the folder structure, do a standard git repo init with remote origin. 

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
- `nb edit [daily, monthly, yearly, someday]`   - Opens the corrisponding note in $EDITOR.
- `nb sync`                                     - Syncs to and from remote git repository.

Configuration
-------------
- Currently defaults to `~/notebook` for notebook root.
- Uses $EDITOR for editor to use with edit command. 
- If $EDITOR is unset, defaults to `nvim`
