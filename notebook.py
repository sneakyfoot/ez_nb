import os, sys, argparse, subprocess, re
from datetime import date, datetime, timedelta
from pathlib import Path
from typing import Optional
from search import search_notebook

notebook_dir = Path("~/notebook").expanduser()
COMPLETED_TASK_PATTERN = re.compile(r"^\s*(?:[-+*]\s+)?\[[xX]\]\s")
INCOMPLETE_TASK_PATTERN = re.compile(r"^\s*(?:[-+*]\s+)?\[\s\]\s*")

def note_header(note_type: str, current: date) -> str:
    if note_type == "daily":
        return f"## {current.strftime('%A %y-%m-%d')}\n\n"
    if note_type == "monthly":
        return f"## {current.strftime('%B %Y')}\n\n"
    if note_type == "yearly":
        return f"## {current.strftime('%Y')}\n\n"
    return ""


def prepare_note(note_type: str, current: date, previous: date, fmt: str) -> Path:
    note_dir = notebook_dir / note_type
    note_today = note_dir / f"{current.strftime(fmt)}.md"
    note_previous = note_dir / f"{previous.strftime(fmt)}.md"

    note_dir.mkdir(parents=True, exist_ok=True)

    if note_today.exists():
        return note_today

    fallback_note = note_previous if note_previous.exists() else find_latest_note(note_dir, current, fmt, note_today)

    if fallback_note and fallback_note.exists():
        previous_content = fallback_note.read_text()
        lines = previous_content.splitlines()
        cleaned_lines = []
        header_skipped = False
        skip_blank_after_header = False

        for line in lines:
            if not header_skipped and line.startswith("## "):
                header_skipped = True
                skip_blank_after_header = True
                continue

            if skip_blank_after_header and line.strip() == "":
                skip_blank_after_header = False
                continue

            skip_blank_after_header = False

            if COMPLETED_TASK_PATTERN.match(line):
                continue

            cleaned_lines.append(line)

        while cleaned_lines and cleaned_lines[0].strip() == "":
            cleaned_lines.pop(0)

        filtered_content = "\n".join(cleaned_lines).rstrip()

        content = note_header(note_type, current)
        if filtered_content:
            content += f"{filtered_content}\n"
        note_today.write_text(content)
    else:
        note_today.write_text(note_header(note_type, current))

    return note_today


def find_latest_note(note_dir: Path, current: date, fmt: str, note_today: Path) -> Optional[Path]:
    latest_path: Optional[Path] = None
    latest_date: Optional[date] = None
    for candidate in note_dir.glob("*.md"):
        if candidate == note_today:
            continue
        try:
            candidate_date = datetime.strptime(candidate.stem, fmt).date()
        except ValueError:
            continue
        if candidate_date >= current:
            continue
        if latest_date is None or candidate_date > latest_date:
            latest_date = candidate_date
            latest_path = candidate
    return latest_path


def collect_uncompleted_tasks(root: Path):
    tasks = []
    for note_path in sorted(root.rglob("*.md")):
        try:
            lines = note_path.read_text(encoding="utf-8").splitlines()
        except OSError:
            continue
        for idx, line in enumerate(lines, start=1):
            if COMPLETED_TASK_PATTERN.match(line):
                continue
            if INCOMPLETE_TASK_PATTERN.match(line):
                tasks.append((note_path, idx, line.strip()))
    return tasks


def list_uncompleted_tasks(root: Path):
    tasks = collect_uncompleted_tasks(root)
    if not tasks:
        print("No uncompleted tasks found.")
        return
    for note_path, line_number, content in tasks:
        try:
            relative = note_path.relative_to(root)
        except ValueError:
            relative = note_path
        print(f"{relative}:{line_number}: {content}")


def resolve_current_notes():
    today = date.today()
    yesterday = today - timedelta(days=1)
    daily_note = prepare_note("daily", today, yesterday, "%y-%m-%d")

    first_of_month = today.replace(day=1)
    previous_month = (first_of_month - timedelta(days=1)).replace(day=1)
    monthly_note = prepare_note("monthly", first_of_month, previous_month, "%y-%m")

    this_year = today.replace(month=1, day=1)
    last_year = this_year.replace(year=this_year.year - 1)
    yearly_note = prepare_note("yearly", this_year, last_year, "%Y")

    someday_note = get_someday_note()

    return [
        ("daily", daily_note),
        ("monthly", monthly_note),
        ("yearly", yearly_note),
        ("someday", someday_note),
    ]


def print_current_notes():
    notes = resolve_current_notes()
    for label, note_path in notes:
        try:
            content = note_path.read_text(encoding="utf-8")
        except OSError:
            content = ""
        try:
            relative = note_path.relative_to(notebook_dir)
        except ValueError:
            relative = note_path
        print(f"--- {label}: {relative} ---")
        if content:
            end = "" if content.endswith("\n") else "\n"
            print(content, end=end)
        print()


def append_to_note(note_path: Path, content: str) -> bool:
    content = content.rstrip("\n")
    if not content.strip():
        print("No content provided to append.", file=sys.stderr)
        return False

    entry = content
    existing = note_path.read_text(encoding="utf-8") if note_path.exists() else ""

    if not existing:
        prefix = ""
    elif existing.endswith("\n\n"):
        prefix = ""
    elif existing.endswith("\n"):
        prefix = "\n"
    else:
        prefix = "\n\n"

    with note_path.open("a", encoding="utf-8") as handle:
        handle.write(f"{prefix}{entry}\n")

    return True


def open_editor(note_path: Path):
    subprocess.run(["nvim", str(note_path)])


def daily():
    today = date.today()
    yesterday = today - timedelta(days=1)
    note_path = prepare_note("daily", today, yesterday, "%y-%m-%d")
    open_editor(note_path)
    return note_path


def monthly():
    today = date.today().replace(day=1)
    previous_month = (today - timedelta(days=1)).replace(day=1)
    note_path = prepare_note("monthly", today, previous_month, "%y-%m")
    open_editor(note_path)
    return note_path


def yearly():
    this_year = date.today().replace(month=1, day=1)
    last_year = this_year.replace(year=this_year.year - 1)
    note_path = prepare_note("yearly", this_year, last_year, "%Y")
    open_editor(note_path)
    return note_path

def get_someday_note() -> Path:
    notebook_dir.mkdir(parents=True, exist_ok=True)
    someday_note = notebook_dir / "someday.md"
    if not someday_note.exists():
        someday_note.touch()
    return someday_note


def someday():
    note_path = get_someday_note()
    open_editor(note_path)
    return note_path


def sync():
    try:
        status = subprocess.run(
            ["git", "status", "--porcelain"],
            cwd=notebook_dir,
            check=True,
            capture_output=True,
            text=True,
        )
    except subprocess.CalledProcessError:
        print("Failed to inspect git status.")
        return

    has_changes = bool(status.stdout.strip())

    if has_changes:
        try:
            subprocess.run(["git", "add", "--all"], cwd=notebook_dir, check=True)
            commit_message = f"Sync {date.today():%Y-%m-%d}"
            subprocess.run(
                ["git", "commit", "-m", commit_message], cwd=notebook_dir, check=True
            )
        except subprocess.CalledProcessError:
            print("Failed to commit notebook changes.")
            return
    else:
        print("No local changes to commit.")

    try:
        subprocess.run(["git", "pull", "--rebase"], cwd=notebook_dir, check=True)
        subprocess.run(["git", "push"], cwd=notebook_dir, check=True)
    except subprocess.CalledProcessError:
        print("Failed to sync with remote.")

def main():
    parser = argparse.ArgumentParser(description="Notebook manager")
    group = parser.add_mutually_exclusive_group()
    group.add_argument("-d", "--daily", action="store_true", help="Open today's note")
    group.add_argument("-m", "--monthly", action="store_true", help="Open this month's note")
    group.add_argument("-y", "--yearly", action="store_true", help="Open this year's note")
    group.add_argument("-s", "--someday", action="store_true", help="Open the someday note")
    group.add_argument("-q", "--search", metavar="QUERY", help="Search notebook contents")
    parser.add_argument(
        "--append",
        nargs="?",
        const="",
        help="Append text to the selected note instead of opening it",
    )
    parser.add_argument(
        "--search-scope",
        choices=["root", "daily", "monthly", "yearly"],
        nargs="+",
        help="Limit search to specific sections",
    )
    parser.add_argument(
        "--search-regex", action="store_true", help="Treat the search query as a regular expression"
    )
    parser.add_argument(
        "--search-case-sensitive",
        action="store_true",
        help="Perform a case-sensitive search",
    )
    parser.add_argument("--sync", action="store_true", help="Sync notebook changes with remote")
    parser.add_argument("--tasks", action="store_true", help="List uncompleted tasks across notes")
    parser.add_argument(
        "--current",
        action="store_true",
        help="Print the contents of the current daily, monthly, yearly, and someday notes",
    )
    args = parser.parse_args()

    append_text = args.append
    append_mode = append_text is not None

    if append_mode and args.search:
        parser.error("--append cannot be used together with --search")

    if append_mode and not (args.daily or args.monthly or args.yearly or args.someday):
        parser.error("--append requires one of --daily, --monthly, --yearly, or --someday")

    if append_mode and append_text == "":
        if sys.stdin.isatty():
            print("Enter text to append (Ctrl-D to finish):", file=sys.stderr)
        append_text = sys.stdin.read()

    opened = False

    if args.tasks:
        list_uncompleted_tasks(notebook_dir)
        opened = True

    if args.current:
        print_current_notes()
        opened = True

    if args.daily:
        today = date.today()
        yesterday = today - timedelta(days=1)
        note_path = prepare_note("daily", today, yesterday, "%y-%m-%d")
        if append_mode:
            if append_to_note(note_path, append_text or ""):
                print(f"Appended entry to {note_path}")
        else:
            open_editor(note_path)
        opened = True
    elif args.monthly:
        today = date.today().replace(day=1)
        previous_month = (today - timedelta(days=1)).replace(day=1)
        note_path = prepare_note("monthly", today, previous_month, "%y-%m")
        if append_mode:
            if append_to_note(note_path, append_text or ""):
                print(f"Appended entry to {note_path}")
        else:
            open_editor(note_path)
        opened = True
    elif args.yearly:
        this_year = date.today().replace(month=1, day=1)
        last_year = this_year.replace(year=this_year.year - 1)
        note_path = prepare_note("yearly", this_year, last_year, "%Y")
        if append_mode:
            if append_to_note(note_path, append_text or ""):
                print(f"Appended entry to {note_path}")
        else:
            open_editor(note_path)
        opened = True
    elif args.someday:
        note_path = get_someday_note()
        if append_mode:
            if append_to_note(note_path, append_text or ""):
                print(f"Appended entry to {note_path}")
        else:
            open_editor(note_path)
        opened = True
    elif args.search:
        result = search_notebook(
            query=args.search,
            root=notebook_dir,
            scopes=args.search_scope,
            regex=args.search_regex,
            case_sensitive=args.search_case_sensitive,
        )
        if result.exit_code == 1:
            print("No matches found.")
        elif result.exit_code not in (0, 1):
            print("Search failed.")
        opened = True

    if not opened and not args.sync:
        print("something else")

    if args.sync:
        sync()

if __name__ == "__main__":
    main()
