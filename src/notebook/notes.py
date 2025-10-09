"""
Core notebook utilities used by the :mod:`notebook` command line interface.
"""

from __future__ import annotations

import os
import re
import shlex
import subprocess
import sys
from contextlib import suppress
from dataclasses import dataclass
from datetime import date, datetime, timedelta
from pathlib import Path
from typing import Iterable, List, Optional, Tuple

COMPLETED_TASK_PATTERN = re.compile(r"^\s*(?:[-+*]\s+)?\[[xX]\]\s")
INCOMPLETE_TASK_PATTERN = re.compile(r"^\s*(?:[-+*]\s+)?\[\s\]\s*")


@dataclass(frozen=True)
class NoteSpec:
    name: str
    stamp: str
    title: str


NOTE_SPECS = {
    "daily": NoteSpec("daily", "%y-%m-%d", "{weekday} {stamp}"),
    "monthly": NoteSpec("monthly", "%y-%m", "{month} {year}"),
    "yearly": NoteSpec("yearly", "%Y", "{year}"),
}


def default_notebook_dir() -> Path:
    """Return the default notebook directory."""
    return Path(os.environ.get("EZ_NB_ROOT", "~/notebook")).expanduser()


def current_timestamp(spec: NoteSpec, reference: date | None = None) -> Tuple[date, str]:
    """Return the current note date and rendered filename stem for a spec."""
    current = (reference or date.today())
    formatted = current.strftime(spec.stamp)
    return current, formatted


def previous_timestamp(spec: NoteSpec, current: date) -> Tuple[date, str]:
    """Return the previous note date and rendered filename stem for a spec."""
    if spec.name == "daily":
        previous = current - timedelta(days=1)
    elif spec.name == "monthly":
        previous = (current.replace(day=1) - timedelta(days=1)).replace(day=1)
    elif spec.name == "yearly":
        previous = current.replace(year=current.year - 1)
    else:
        previous = current
    return previous, previous.strftime(spec.stamp)


def note_header(spec: NoteSpec, note_date: date) -> str:
    """Render the Markdown heading placed at the top of a newly created note."""
    if spec.name == "daily":
        return f"## {note_date.strftime('%A %y-%m-%d')}\n\n"
    if spec.name == "monthly":
        return f"## {note_date.strftime('%B %Y')}\n\n"
    if spec.name == "yearly":
        return f"## {note_date.strftime('%Y')}\n\n"
    return ""


def _find_latest_note(
    note_dir: Path,
    current_date: date,
    spec: NoteSpec,
    today_path: Path,
) -> Optional[Path]:
    latest_path: Optional[Path] = None
    latest_date: Optional[date] = None

    for candidate in note_dir.glob("*.md"):
        if candidate == today_path:
            continue
        with suppress(ValueError):
            candidate_date = datetime.strptime(candidate.stem, spec.stamp).date()
            if candidate_date >= current_date:
                continue
            if latest_date is None or candidate_date > latest_date:
                latest_date = candidate_date
                latest_path = candidate
    return latest_path


def prepare_note(
    spec: NoteSpec,
    notebook_dir: Path,
    reference_date: date | None = None,
) -> Path:
    """
    Ensure the note for ``reference_date`` exists and carry over unfinished items.

    Notes are stored under ``<notebook_dir>/<spec.name>/``. When a fresh note is
    created the most recent previous note is inspected and any completed tasks
    (marked ``[x]``) are removed while unfinished tasks and free-form text are
    preserved.
    """

    current_date, current_stamp = current_timestamp(spec, reference_date)
    note_dir = notebook_dir / spec.name
    note_today = note_dir / f"{current_stamp}.md"

    note_dir.mkdir(parents=True, exist_ok=True)

    if note_today.exists():
        return note_today

    previous_date, previous_stamp = previous_timestamp(spec, current_date)
    note_previous = note_dir / f"{previous_stamp}.md"

    fallback_note = (
        note_previous
        if note_previous.exists()
        else _find_latest_note(note_dir, current_date, spec, note_today)
    )

    if fallback_note and fallback_note.exists():
        previous_content = fallback_note.read_text(encoding="utf-8")
        lines = previous_content.splitlines()
        cleaned_lines: List[str] = []
        header_skipped = False
        skip_blank_after_header = False

        for line in lines:
            if not header_skipped and line.startswith("## "):
                header_skipped = True
                skip_blank_after_header = True
                continue

            if skip_blank_after_header and not line.strip():
                skip_blank_after_header = False
                continue

            skip_blank_after_header = False

            if COMPLETED_TASK_PATTERN.match(line):
                continue

            cleaned_lines.append(line)

        while cleaned_lines and not cleaned_lines[0].strip():
            cleaned_lines.pop(0)

        filtered_content = "\n".join(cleaned_lines).rstrip()
        content = note_header(spec, current_date)
        if filtered_content:
            content += f"{filtered_content}\n"
        note_today.write_text(content, encoding="utf-8")
    else:
        note_today.write_text(note_header(spec, current_date), encoding="utf-8")

    return note_today


def get_someday_note(notebook_dir: Path) -> Path:
    notebook_dir.mkdir(parents=True, exist_ok=True)
    someday_note = notebook_dir / "someday.md"
    if not someday_note.exists():
        someday_note.touch()
    return someday_note


def collect_uncompleted_tasks(root: Path) -> List[Tuple[Path, int, str]]:
    """Return a list of unchecked Markdown tasks inside ``root``."""
    tasks: List[Tuple[Path, int, str]] = []
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


def list_uncompleted_tasks(root: Path) -> None:
    tasks = collect_uncompleted_tasks(root)
    if not tasks:
        print("No uncompleted tasks found.")
        return
    for note_path, line_number, content in tasks:
        display_path: Path = note_path
        with suppress(ValueError):
            display_path = note_path.relative_to(root)
        print(f"{display_path}:{line_number}: {content}")


def resolve_current_notes(notebook_dir: Path) -> List[Tuple[str, Path]]:
    today = date.today()
    notes = []
    for key, spec in NOTE_SPECS.items():
        note_path = prepare_note(spec, notebook_dir, reference_date=today)
        notes.append((key, note_path))
    notes.append(("someday", get_someday_note(notebook_dir)))
    return notes


def print_current_notes(notebook_dir: Path) -> None:
    notes = resolve_current_notes(notebook_dir)
    for label, note_path in notes:
        display_path: Path = note_path
        with suppress(ValueError):
            display_path = note_path.relative_to(notebook_dir)
        content = ""
        with suppress(OSError):
            content = note_path.read_text(encoding="utf-8")
        print(f"--- {label}: {display_path} ---")
        if content:
            end = "" if content.endswith("\n") else "\n"
            print(content, end=end)
        print()


def append_to_note(note_path: Path, content: str) -> bool:
    content = content.rstrip("\n")
    if not content.strip():
        print("No content provided to append.", file=sys.stderr)
        return False

    existing = ""
    if note_path.exists():
        existing = note_path.read_text(encoding="utf-8")

    if not existing:
        prefix = ""
    elif existing.endswith("\n\n"):
        prefix = ""
    elif existing.endswith("\n"):
        prefix = "\n"
    else:
        prefix = "\n\n"

    entry = f"{prefix}{content}\n"

    with note_path.open("a", encoding="utf-8") as handle:
        handle.write(entry)

    return True


def open_editor(note_path: Path, editor: Optional[str] = None) -> None:
    command = (editor or os.environ.get("EDITOR") or "nvim").strip()
    if not command:
        command = "nvim"
    parts = shlex.split(command)
    try:
        subprocess.run([*parts, str(note_path)], check=False)
    except FileNotFoundError:
        print(f"Editor command not found: {parts[0]}", file=sys.stderr)
    except OSError as exc:
        print(f"Failed to launch editor {parts[0]!r}: {exc}", file=sys.stderr)


def open_note_interactively(
    spec_name: str,
    notebook_dir: Path,
    append_text: Optional[str],
    editor: Optional[str] = None,
) -> Optional[Path]:
    if spec_name == "someday":
        note_path = get_someday_note(notebook_dir)
    else:
        spec = NOTE_SPECS[spec_name]
        note_path = prepare_note(spec, notebook_dir)

    if append_text is not None:
        if append_to_note(note_path, append_text or ""):
            print(f"Appended entry to {note_path}")
        return note_path

    open_editor(note_path, editor=editor)
    return note_path


def sync_notebook(notebook_dir: Path) -> None:
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
