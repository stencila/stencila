#!/usr/bin/env python3
"""Print shell exports for this analyzer in any resolved consumer checkout."""
import argparse
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--manifest-path", type=Path, default=Path(__file__).resolve().parents[1] / "Cargo.toml")
parser.add_argument("--offline", action="store_true")
args = parser.parse_args()
command = ["cargo", "metadata", "--locked", "--format-version", "1", "--manifest-path", str(args.manifest_path), "--filter-platform", "wasm32-unknown-unknown"]
if args.offline:
    command.append("--offline")
metadata = json.loads(subprocess.check_output(command))
packages = metadata["packages"]

def package(name, version=None):
    matches = [item for item in packages if item["name"] == name and (version is None or item["version"] == version)]
    if len(matches) != 1:
        raise SystemExit(f"Expected one resolved {name} package, found {len(matches)}")
    return Path(matches[0]["manifest_path"]).parent

include = package("tree-sitter-language", "0.1.7") / "wasm/include"
compat = package("stencila-code-analysis") / "include/compat.h"
if not include.is_dir() or not compat.is_file():
    raise SystemExit("The resolved analyzer's wasm compatibility headers are missing")
cc = os.environ.get("CC_wasm32_unknown_unknown", "clang-18")
ar = os.environ.get("AR_wasm32_unknown_unknown", "llvm-ar-18")
if not shutil.which(cc) or not shutil.which(ar):
    raise SystemExit("Install clang-18 and llvm-18, or set target-specific CC/AR")
version = subprocess.check_output([cc, "--version"], text=True).splitlines()[0]
if "clang version 18.1.3" not in version:
    raise SystemExit(f"Supported compiler is clang 18.1.3; found {version}")
flags = shlex.join(["-I", str(include), "-include", str(compat)])
extra = os.environ.get("CFLAGS_wasm32_unknown_unknown", "")
if extra:
    flags += " " + extra
for name, value in {
    "CC_wasm32_unknown_unknown": cc,
    "AR_wasm32_unknown_unknown": ar,
    "CFLAGS_wasm32_unknown_unknown": flags,
}.items():
    print(f"export {name}={shlex.quote(value)}")
