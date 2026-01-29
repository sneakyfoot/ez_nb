"""
Wrapper utilities around ``rg`` to search notebook contents.
"""

from __future__ import annotations

import subprocess
from pathlib import Path
from typing import Iterable, List, Optional


class SearchResult:
    def __init__(self, matched: bool, exit_code: int):
        self.matched = matched
        self.exit_code = exit_code


def _build_search_paths(root: Path, scopes: Optional[Iterable[str]]) -> List[Path]:
    if not scopes:
        return [root]

    scope_map = {
        "root": root,
        "daily": root / "daily",
        "monthly": root / "monthly",
        "yearly": root / "yearly",
    }
    paths: List[Path] = []
    for scope in scopes:
        target = scope_map.get(scope)
        if target and target.exists():
            paths.append(target)
    # fallback to root if nothing resolved
    return paths or [root]


def _run_ripgrep(
    query: str,
    paths: Iterable[Path],
    root: Path,
    regex: bool,
    case_sensitive: bool,
) -> SearchResult:
    cmd = ["rg", "--with-filename", "--line-number", "-g", "*.md"]

    if not regex:
        cmd.append("--fixed-strings")
    if not case_sensitive:
        cmd.append("--ignore-case")

    cmd.append(query)

    for path in paths:
        try:
            relative = path.relative_to(root)
            cmd.append(str(relative) or ".")
        except ValueError:
            cmd.append(str(path))

    try:
        result = subprocess.run(cmd, cwd=root, check=False)
        # ripgrep returns 0 for matches, 1 for no matches
        matched = result.returncode == 0
        return SearchResult(matched=matched, exit_code=result.returncode)
    except FileNotFoundError:
        print("ripgrep (rg) is required for search, but it was not found on PATH.")
        return SearchResult(matched=False, exit_code=127)


def search_notebook(
    query: str,
    root: Path,
    scopes: Optional[Iterable[str]] = None,
    regex: bool = False,
    case_sensitive: bool = False,
) -> SearchResult:
    paths = _build_search_paths(root, scopes)

    if not root.exists():
        print(f"Notebook directory {root} does not exist.")
        return SearchResult(matched=False, exit_code=2)

    return _run_ripgrep(query, paths, root, regex, case_sensitive)
