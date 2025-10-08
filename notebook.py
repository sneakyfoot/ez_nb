import os, sys, argparse, subprocess
from datetime import date, timedelta
from pathlib import Path

notebook_dir = Path("~/notebook").expanduser()

def note_header(note_type: str, current: date) -> str:
    if note_type == "daily":
        return f"## {current.strftime('%A %y-%m-%d')}\n\n"
    if note_type == "monthly":
        return f"## {current.strftime('%B %Y')}\n\n"
    if note_type == "yearly":
        return f"## {current.strftime('%Y')}\n\n"
    return ""


def open_note(note_type: str, current: date, previous: date, fmt: str):
    note_dir = notebook_dir / note_type
    note_today = note_dir / f"{current.strftime(fmt)}.md"
    note_previous = note_dir / f"{previous.strftime(fmt)}.md"

    note_dir.mkdir(parents=True, exist_ok=True)

    if note_today.exists():
        subprocess.run(["nvim", str(note_today)])
        return

    if note_previous.exists():
        note_today.write_text(note_previous.read_text())
    else:
        note_today.write_text(note_header(note_type, current))

    subprocess.run(["nvim", str(note_today)])


def daily():
    today = date.today()
    yesterday = today - timedelta(days=1)
    open_note("daily", today, yesterday, "%y-%m-%d")


def monthly():
    today = date.today().replace(day=1)
    previous_month = (today - timedelta(days=1)).replace(day=1)
    open_note("monthly", today, previous_month, "%y-%m")


def yearly():
    this_year = date.today().replace(month=1, day=1)
    last_year = this_year.replace(year=this_year.year - 1)
    open_note("yearly", this_year, last_year, "%Y")


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
    parser.add_argument("--sync", action="store_true", help="Sync notebook changes with remote")
    args = parser.parse_args()

    opened = False

    if args.daily:
        daily()
        opened = True
    elif args.monthly:
        monthly()
        opened = True
    elif args.yearly:
        yearly()
        opened = True

    if not opened and not args.sync:
        print("something else")

    if args.sync:
        sync()

if __name__ == "__main__":
    main()
