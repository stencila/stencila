#!/usr/bin/env python3
"""Reject repository paths that cannot be checked out on Windows."""

from pathlib import Path
import re
import subprocess
import sys


def main() -> int:
    root = subprocess.check_output(
        ["git", "rev-parse", "--show-toplevel"], text=True
    ).strip()
    paths = subprocess.check_output(
        ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
        cwd=root,
    ).decode().split("\0")
    invalid = []
    for path in paths:
        # Include uncommitted renames without checking the removed source path.
        local = Path(root, path)
        if not path or not (local.exists() or local.is_symlink()):
            continue
        for part in path.split("/"):
            if (
                re.search(r'[<>:"\\|?*\x00-\x1f]', part)
                or part.endswith((" ", "."))
                or re.match(r"^(CON|PRN|AUX|NUL|COM[1-9¹²³]|LPT[1-9¹²³])(\.|$)", part, re.I)
            ):
                invalid.append(path)
                break

    if invalid:
        print("Paths incompatible with Windows checkout:", file=sys.stderr)
        for path in sorted(set(invalid)):
            print(f"  {path}", file=sys.stderr)
        return 1

    print("All repository paths are compatible with Windows checkout.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
