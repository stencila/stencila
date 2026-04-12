#!/usr/bin/env bash
set -euo pipefail

# Probe GitHub pull request review anchor acceptance using a real repository.
#
# This script creates a temporary branch and PR in the current GitHub repository,
# then measures which source lines GitHub accepts for inline PR review comments
# relative to a single changed line.
#
# Why this exists
# ---------------
# The GitHub API documentation is ambiguous about how far away from the nearest
# changed diff hunk an inline review comment may be anchored. In practice,
# comments that GitHub cannot anchor are rejected with HTTP 422 responses such as:
#
#   - "Pull request review comment line must be part of the diff"
#   - "Pull request review comment position is invalid"
#   - other diff-anchor validation failures
#
# The `stencila-codec-github` crate has logic to detect and recover from these
# cases. This script provides an empirical way to probe current GitHub behavior
# against a real repository before encoding assumptions in Rust tests.
#
# What the script does
# --------------------
# 1. Creates a temporary branch from a chosen base branch.
# 2. Writes a numbered fixture file with 50 lines: 01..50.
# 3. Commits that file.
# 4. Modifies exactly one line (default: line 25) and commits again.
# 5. Opens a temporary pull request.
# 6. Attempts to create single inline PR review comments on a series of candidate
#    lines around the changed line.
# 7. Prints which lines GitHub accepted and which were rejected, including the
#    raw API error output for failures.
#
# Why use direct review comments instead of a full review payload?
# ---------------------------------------------------------------
# The script uses:
#   POST /repos/{owner}/{repo}/pulls/{pull_number}/comments
#
# rather than batching multiple comments into a single review. This isolates each
# candidate line so that one invalid anchor does not invalidate the whole batch.
# That makes the probe much more useful for understanding GitHub's behavior.
#
# Requirements
# ------------
# - Run inside a Git repository hosted on GitHub.
# - `gh` (GitHub CLI) installed and authenticated.
# - `jq` installed.
# - `python3` installed.
# - Push permission to the repository.
#
# Compatibility note
# ------------------
# Some `gh` versions do not support `gh pr create --json ...`. This script
# avoids that flag and instead captures the created PR URL from stdout, then
# queries `gh pr view` to retrieve structured metadata.
#
# Usage
# -----
#   ./rust/codec-github/tests/probe-pr-review-distance.sh [base-branch]
#
# Examples
# --------
#   ./rust/codec-github/tests/probe-pr-review-distance.sh main
#   CHANGE_LINE=25 ./rust/codec-github/tests/probe-pr-review-distance.sh main
#   FILE_PATH=tmp/review-anchor-distance.txt \
#     ./rust/codec-github/tests/probe-pr-review-distance.sh main
#   AUTO_CLEANUP=1 ./rust/codec-github/tests/probe-pr-review-distance.sh main
#
# Environment variables
# ---------------------
# REMOTE         Git remote to use (default: origin)
# FILE_PATH      Path of temporary probe file in the repo
#                (default: review-anchor-distance.txt)
# CHANGE_LINE    One-based changed line in the 50-line fixture
#                (default: 25)
# MODE           Probe mode: `new-file` or `existing-file`
#                - new-file: the file is introduced by the PR
#                - existing-file: the file is first committed to the base branch,
#                  then the PR changes a single line in that pre-existing file
#                (default: existing-file)
# PREPARE_BASE   In `existing-file` mode, if set to 1 and the fixture file does
#                not exist on the base branch, commit it to the base branch and
#                push that commit before creating the probe PR
#                (default: 0)
# AUTO_CLEANUP   If set to 1, close the PR and delete the branch at the end
#                (default: 0)
#
# Outputs
# -------
# The script prints:
# - repo, branch, PR URL, and head SHA
# - accepted candidate lines
# - rejected candidate lines
# - raw GitHub API error output for rejected lines
#
# Interpreting results
# --------------------
# The most important outcomes are:
# - whether the changed line is accepted
# - whether nearby unchanged lines are accepted
# - how far from the diff GitHub starts rejecting comments with 422 errors
#
# These results can then inform Rust tests in `codec-github`, especially tests
# around recognizing and handling diff-anchor related 422 responses.

BASE_BRANCH="${1:-main}"
REMOTE="${REMOTE:-origin}"
FILE_PATH="${FILE_PATH:-review-anchor-distance.txt}"
CHANGE_LINE="${CHANGE_LINE:-25}"
MODE="${MODE:-existing-file}"
PREPARE_BASE="${PREPARE_BASE:-0}"
AUTO_CLEANUP="${AUTO_CLEANUP:-0}"

if ! command -v gh >/dev/null 2>&1; then
  echo "Error: gh is required" >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "Error: jq is required" >&2
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "Error: python3 is required" >&2
  exit 1
fi

REPO="$(gh repo view --json nameWithOwner -q .nameWithOwner)"
OWNER="${REPO%/*}"
NAME="${REPO#*/}"

STAMP="$(date +%Y%m%d-%H%M%S)"
BRANCH="probe/review-anchor-distance-$STAMP"

CANDIDATE_LINES=(25 24 26 23 27 22 28 21 29 20 30 15 35 10 40 1 50)

cleanup() {
  set +e
  git checkout "$BASE_BRANCH" >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "Repo:         $REPO"
echo "Base branch:  $BASE_BRANCH"
echo "Temp branch:  $BRANCH"
echo "Mode:         $MODE"
echo "File:         $FILE_PATH"
echo "Changed line: $CHANGE_LINE"
echo

echo "Fetching base branch..."
git fetch "$REMOTE" "$BASE_BRANCH"
git checkout -B "$BASE_BRANCH" "$REMOTE/$BASE_BRANCH"

echo "Creating temp branch..."
git checkout -b "$BRANCH"

write_numbered_file() {
  mkdir -p "$(dirname "$FILE_PATH")"
  : > "$FILE_PATH"
  for i in $(seq 1 50); do
    printf "%02d\n" "$i" >> "$FILE_PATH"
  done
}

if [[ "$MODE" == "new-file" ]]; then
  echo "Writing numbered file in PR branch..."
  write_numbered_file

  git add "$FILE_PATH"
  git commit -m "test: add numbered file for review anchor probe"
elif [[ "$MODE" == "existing-file" ]]; then
  echo "Checking whether fixture file exists on base branch..."
  if git cat-file -e "$REMOTE/$BASE_BRANCH:$FILE_PATH" 2>/dev/null; then
    echo "Fixture file already exists on base branch; checking it out into temp branch..."
    git checkout "$REMOTE/$BASE_BRANCH" -- "$FILE_PATH"
  else
    if [[ "$PREPARE_BASE" != "1" ]]; then
      echo "Error: $FILE_PATH does not exist on $BASE_BRANCH." >&2
      echo "Either:" >&2
      echo "  - create it manually on $BASE_BRANCH first, or" >&2
      echo "  - rerun with PREPARE_BASE=1 to have this script commit it to $BASE_BRANCH" >&2
      exit 1
    fi

    echo "Preparing base branch with fixture file..."
    git checkout "$BASE_BRANCH"
    write_numbered_file
    git add "$FILE_PATH"
    git commit -m "test: add numbered file for review anchor probe"
    git push "$REMOTE" "$BASE_BRANCH"

    echo "Recreating temp branch from updated base branch..."
    git checkout -B "$BRANCH" "$REMOTE/$BASE_BRANCH"
  fi
else
  echo "Error: MODE must be 'new-file' or 'existing-file'" >&2
  exit 1
fi

echo "Changing line $CHANGE_LINE..."
python3 - <<'PY' "$FILE_PATH" "$CHANGE_LINE"
import sys
path = sys.argv[1]
change_line = int(sys.argv[2])
with open(path, "r", encoding="utf-8") as f:
    lines = f.read().splitlines()
if not (1 <= change_line <= len(lines)):
    raise SystemExit(f"CHANGE_LINE {change_line} out of range for {len(lines)}-line fixture")
lines[change_line - 1] = f"{change_line:02d} changed"
with open(path, "w", encoding="utf-8", newline="\n") as f:
    for line in lines:
        f.write(line + "\n")
PY

git add "$FILE_PATH"
git commit -m "test: change one line for review anchor probe"

echo "Pushing branch..."
git push -u "$REMOTE" "$BRANCH"

echo "Creating PR..."
PR_CREATE_OUTPUT="$(gh pr create \
  --base "$BASE_BRANCH" \
  --head "$BRANCH" \
  --title "test: probe review anchor distance $STAMP" \
  --body "Temporary PR to probe GitHub PR review comment anchor behavior.")"
PR_URL="$(printf '%s\n' "$PR_CREATE_OUTPUT" | tail -n 1)"

if [[ -z "$PR_URL" || "$PR_URL" != https://github.com/*/pull/* ]]; then
  echo "Error: unable to determine PR URL from gh pr create output:" >&2
  printf '%s\n' "$PR_CREATE_OUTPUT" >&2
  exit 1
fi

PR_JSON="$(gh pr view "$PR_URL" --json number,url,headRefOid)"
PR_NUMBER="$(jq -r .number <<<"$PR_JSON")"
HEAD_SHA="$(jq -r .headRefOid <<<"$PR_JSON")"

echo "PR: $PR_URL"
echo "PR number: $PR_NUMBER"
echo "Head SHA: $HEAD_SHA"
echo

api_post_review_comment() {
  local line="$1"
  local body="$2"

  gh api \
    -X POST \
    -H "Accept: application/vnd.github+json" \
    "/repos/$OWNER/$NAME/pulls/$PR_NUMBER/comments" \
    -f body="$body" \
    -f commit_id="$HEAD_SHA" \
    -f path="$FILE_PATH" \
    -F line="$line" \
    -f side="RIGHT" 2>&1
}

echo "Probing review comment anchors..."
echo

ACCEPTED=()
REJECTED=()

for line in "${CANDIDATE_LINES[@]}"; do
  echo "== line $line =="
  set +e
  OUTPUT="$(api_post_review_comment "$line" "Probe comment on line $line")"
  STATUS=$?
  set -e

  if [ "$STATUS" -eq 0 ]; then
    echo "accepted"
    ACCEPTED+=("$line")
  else
    echo "rejected"
    echo "$OUTPUT"
    REJECTED+=("$line")
  fi
  echo
done

echo "Summary"
echo "-------"
echo "Accepted: ${ACCEPTED[*]:-<none>}"
echo "Rejected: ${REJECTED[*]:-<none>}"
echo
echo "PR kept open for inspection:"
echo "  $PR_URL"
echo

if [ "$AUTO_CLEANUP" = "1" ]; then
  echo "Cleaning up PR and branch..."
  gh pr close "$PR_NUMBER" --delete-branch
else
  echo "To clean up later:"
  echo "  gh pr close $PR_NUMBER --delete-branch"
fi
