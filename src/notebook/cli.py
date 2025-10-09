"""
Command line interface for the ez-nb notebook helper.
"""

from __future__ import annotations

import argparse
import sys
from importlib import metadata
from pathlib import Path
from typing import Iterable, Optional

from . import __version__ as package_version
from . import notes
from .search import search_notebook


def _package_version() -> str:
    try:
        return metadata.version("notebook")
    except metadata.PackageNotFoundError:
        return package_version


def build_parser() -> argparse.ArgumentParser:
    default_root = notes.default_notebook_dir()
    parser = argparse.ArgumentParser(
        prog="notebook",
        description="Manage rolling Markdown notebook files.",
    )
    parser.add_argument(
        "--root",
        type=Path,
        default=default_root,
        help=f"Notebook directory (default: {default_root})",
    )
    parser.add_argument(
        "--editor",
        help="Command to open notes (falls back to $EDITOR or nvim).",
    )

    note_group = parser.add_mutually_exclusive_group()
    note_group.add_argument("-d", "--daily", action="store_true", help="Open today's note.")
    note_group.add_argument("-m", "--monthly", action="store_true", help="Open this month's note.")
    note_group.add_argument("-y", "--yearly", action="store_true", help="Open this year's note.")
    note_group.add_argument("-s", "--someday", action="store_true", help="Open the someday note.")
    note_group.add_argument("-q", "--search", metavar="QUERY", help="Search notebook contents.")

    parser.add_argument(
        "--append",
        nargs="?",
        const="",
        help="Append text to the selected note instead of opening it.",
    )
    parser.add_argument(
        "--search-scope",
        choices=["root", "daily", "monthly", "yearly"],
        nargs="+",
        help="Limit search to specific sections.",
    )
    parser.add_argument(
        "--search-regex",
        action="store_true",
        help="Treat the search query as a regular expression.",
    )
    parser.add_argument(
        "--search-case-sensitive",
        action="store_true",
        help="Perform a case-sensitive search.",
    )
    parser.add_argument("--sync", action="store_true", help="Sync notebook changes with remote.")
    parser.add_argument("--tasks", action="store_true", help="List uncompleted tasks across notes.")
    parser.add_argument(
        "--current",
        action="store_true",
        help="Print the current daily, monthly, yearly, and someday notes.",
    )
    parser.add_argument(
        "-V",
        "--version",
        action="version",
        version=f"%(prog)s {_package_version()}",
    )
    return parser


def _resolve_note_selection(args: argparse.Namespace) -> Optional[str]:
    if args.daily:
        return "daily"
    if args.monthly:
        return "monthly"
    if args.yearly:
        return "yearly"
    if args.someday:
        return "someday"
    return None


def run_cli(args: argparse.Namespace, parser: argparse.ArgumentParser) -> int:
    append_text = args.append
    append_mode = append_text is not None

    if append_mode and args.search:
        parser.error("--append cannot be used together with --search")

    if append_mode and not (
        args.daily or args.monthly or args.yearly or args.someday
    ):
        parser.error("--append requires one of --daily, --monthly, --yearly, or --someday")

    if append_mode and append_text == "":
        if sys.stdin.isatty():
            print("Enter text to append (Ctrl-D to finish):", file=sys.stderr)
        append_text = sys.stdin.read()

    notebook_dir: Path = Path(args.root).expanduser()
    notebook_dir.mkdir(parents=True, exist_ok=True)

    performed_action = False

    if args.tasks:
        notes.list_uncompleted_tasks(notebook_dir)
        performed_action = True

    if args.current:
        notes.print_current_notes(notebook_dir)
        performed_action = True

    selection = _resolve_note_selection(args)

    if selection:
        notes.open_note_interactively(
            selection,
            notebook_dir=notebook_dir,
            append_text=append_text,
            editor=args.editor,
        )
        performed_action = True
    elif args.search:
        scopes: Optional[Iterable[str]] = args.search_scope
        result = search_notebook(
            query=args.search,
            root=notebook_dir,
            scopes=scopes,
            regex=args.search_regex,
            case_sensitive=args.search_case_sensitive,
        )
        if result.exit_code == 1:
            print("No matches found.")
        elif result.exit_code not in (0, 1):
            print("Search failed.")
        performed_action = True

    if not performed_action and not args.sync:
        parser.print_help()

    if args.sync:
        notes.sync_notebook(notebook_dir)
        performed_action = True

    return 0


def parse_args(argv: Optional[Iterable[str]] = None) -> tuple[argparse.ArgumentParser, argparse.Namespace]:
    parser = build_parser()
    args = parser.parse_args(argv)
    return parser, args


def main(argv: Optional[Iterable[str]] = None) -> int:
    parser, args = parse_args(argv)
    return run_cli(args, parser)


if __name__ == "__main__":
    raise SystemExit(main())
