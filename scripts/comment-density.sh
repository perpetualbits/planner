#!/usr/bin/env bash
# comment-density.sh — audit the comment density of one or more Rust source files.
#
# Usage:
#   scripts/comment-density.sh <file> [<file> ...]
#   scripts/comment-density.sh crates/amity-core/src/inbox.rs
#
# Exit codes:
#   0  All files meet the ≥50% threshold.
#   1  One or more files are below threshold or an argument error occurred.
#
# What counts as a comment line:
#   Lines that, after stripping leading whitespace, start with // or ///.
#   Block comments (/* ... */) are not counted — they are rare in this codebase
#   and handling multi-line blocks correctly in bash is error-prone. If block
#   comments are needed, this script should be replaced with a proper Rust tool.
#
# What counts as a code line:
#   Non-empty, non-blank lines that are not comment lines.
#
# The 50% target means: comment_lines / (comment_lines + code_lines) >= 0.5
# Blank lines are excluded from both counts — they carry no information either way.

set -euo pipefail

# Minimum ratio expressed as an integer percentage (0–100).
THRESHOLD=50

# Track whether any file failed so we can return a non-zero exit code at the end
# without stopping the loop early (we want to report all failing files, not just
# the first one).
any_failed=0

if [[ $# -eq 0 ]]; then
    echo "Usage: $0 <file> [<file> ...]" >&2
    exit 1
fi

for file in "$@"; do
    if [[ ! -f "$file" ]]; then
        echo "ERROR: not a file: $file" >&2
        any_failed=1
        continue
    fi

    # Count comment lines: strip leading whitespace, check for // prefix.
    comment_lines=$(grep -cE '^\s*//' "$file" || true)

    # Count blank lines so we can subtract them from total.
    blank_lines=$(grep -cE '^\s*$' "$file" || true)

    total_lines=$(wc -l < "$file")

    # Code lines = total - blank - comment.
    # The arithmetic uses bash integer arithmetic; no floating point needed here
    # because we compare percentages as integers (multiply by 100 before dividing).
    code_lines=$(( total_lines - blank_lines - comment_lines ))

    # Guard against a file that is entirely comments or entirely blank.
    if [[ $(( comment_lines + code_lines )) -eq 0 ]]; then
        echo "SKIP  $file (no non-blank lines)"
        continue
    fi

    # Integer percentage: (comment_lines * 100) / (comment_lines + code_lines).
    # Integer division truncates, so 49.9% becomes 49 and fails the check.
    # This is intentional — the threshold is a floor, not a target.
    ratio=$(( (comment_lines * 100) / (comment_lines + code_lines) ))

    if [[ $ratio -ge $THRESHOLD ]]; then
        echo "OK    $file  (${ratio}% comments, ${comment_lines} comment / ${code_lines} code)"
    else
        echo "FAIL  $file  (${ratio}% comments, ${comment_lines} comment / ${code_lines} code — need ${THRESHOLD}%)"
        any_failed=1
    fi
done

exit "$any_failed"
