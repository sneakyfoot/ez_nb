import os, sys, argparse, subprocess
from datetime import date, timedelta
from pathlib import Path

notebook_dir = Path("~/notebook").expanduser()

def daily():
    today = date.today()
    yesterday = today - timedelta(days=1)

    today_note = notebook_dir / "daily" / f"{today:%y-%m-%d}.md"
    yesterday_note = notebook_dir / "daily" / f"{yesterday:%y-%m-%d}.md"

    today_note.parent.mkdir(parents=True, exist_ok=True)

    if today_note.exists():
        # open today's note
        subprocess.run(["nvim", str(today_note)])
        return

    elif yesterday_note.exists():
        # copy yesterday's note to today, then open
        today_note.write_text(yesterday_note.read_text())
        subprocess.run(["nvim", str(today_note)])
        return

    else:
        # neither exists, open new file (vim will create it)
        subprocess.run(["nvim", str(today_note)])

def main():
    parser = argparse.ArgumentParser(description="Notebook manager")
    parser.add_argument("-d", "--daily", action="store_true", help="Open Today's note")
    args = parser.parse_args()

    if args.daily:
        daily()
    else:
        print("something else")


if __name__ == "__main__":
    main()
