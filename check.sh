#!/usr/bin/env bash
# check.sh — Quality gate for the RMTFAR workspace.
#
# Steps (all must pass for exit 0):
#   1. rustfmt      — code formatting
#   2. clippy       — lints & correctness
#   3. cargo test   — unit + integration tests
#   4. SQF checks   — static analysis on Arma 3 scripts
#   5. cargo audit  — known CVEs          (skipped if not installed)
#   6. cargo machete — unused dependencies (skipped if not installed)
#
# Usage:
#   ./check.sh            # normal run
#   ./check.sh --fix      # auto-fix formatting before checking
#   QUIET=1 ./check.sh   # suppress sub-command stdout

set -uo pipefail

# ── Options ──────────────────────────────────────────────────────────────────
FIX=0
for arg in "$@"; do
    case "$arg" in
        --fix) FIX=1 ;;
        --help|-h)
            echo "Usage: $0 [--fix]"
            echo "  --fix   Auto-format code before running checks."
            exit 0
            ;;
    esac
done

# ── Colours ──────────────────────────────────────────────────────────────────
if [[ -t 1 ]]; then
    RED=$'\033[0;31m' GRN=$'\033[0;32m' YEL=$'\033[0;33m'
    CYN=$'\033[0;36m' BLD=$'\033[1m'    RST=$'\033[0m'
else
    RED='' GRN='' YEL='' CYN='' BLD='' RST=''
fi

PASS="${GRN}✔ PASS${RST}"
FAIL="${RED}✘ FAIL${RST}"
SKIP="${YEL}– SKIP${RST}"

ERRORS=0
SKIPPED=0
START_TIME=$(date +%s)

banner() { echo; echo "${CYN}${BLD}── $* ──${RST}"; }
ok()     { echo "  ${PASS}  $*"; }
warn()   { echo "  ${YEL}⚠ WARN${RST}  $*"; }
fail()   { echo "  ${FAIL}  $*"; ERRORS=$((ERRORS + 1)); }
skip()   { echo "  ${SKIP}  $*"; SKIPPED=$((SKIPPED + 1)); }

# Run a command, suppress stdout when QUIET=1, always show stderr.
run() {
    if [[ "${QUIET:-0}" == "1" ]]; then
        "$@" 2>&1 | grep -E "^(error|warning\[)" || true
        return "${PIPESTATUS[0]}"
    else
        "$@"
    fi
}

# ── 0. Environment check ─────────────────────────────────────────────────────
banner "Environment"
echo "  Rust  : $(rustc --version)"
echo "  Cargo : $(cargo --version)"
echo "  Host  : $(rustc -vV | grep host | cut -d' ' -f2)"

# ── 1. Formatting ────────────────────────────────────────────────────────────
banner "rustfmt — formatting"
if [[ $FIX -eq 1 ]]; then
    cargo fmt --all
    ok "Auto-formatted (--fix mode)"
elif run cargo fmt --all -- --check; then
    ok "All files are correctly formatted"
else
    fail "Formatting issues found — run:  cargo fmt --all  (or  ./check.sh --fix)"
fi

# ── 2. Clippy ────────────────────────────────────────────────────────────────
banner "clippy — lints"
CLIPPY_FLAGS=(
    -D warnings                        # treat all warnings as errors
    -W clippy::pedantic                # extra pedantic lints
    -A clippy::must_use_candidate      # too noisy for this project
    -A clippy::missing_errors_doc      # docs phase comes later
    -A clippy::missing_panics_doc
    -A clippy::module_name_repetitions # rmtfar_protocol::PlayerState etc.
)
if run cargo clippy --workspace --all-targets -- "${CLIPPY_FLAGS[@]}"; then
    ok "No clippy warnings"
else
    fail "Clippy found issues"
fi

# ── 3. Tests ─────────────────────────────────────────────────────────────────
banner "cargo test — unit + integration tests"
if run cargo test --workspace; then
    ok "All tests passed"
else
    fail "Test failures"
fi

# ── 4. Docs (no broken links) ────────────────────────────────────────────────
banner "cargo doc — documentation"
if run cargo doc --workspace --no-deps --quiet 2>&1 | grep -v "^$"; then
    ok "Documentation builds cleanly"
else
    # cargo doc --quiet gives no output on success; non-zero exit = failure
    if [[ ${PIPESTATUS[0]} -ne 0 ]]; then
        fail "Documentation errors found"
    else
        ok "Documentation builds cleanly"
    fi
fi

# ── 5. SQF static checks ─────────────────────────────────────────────────────
banner "SQF — static analysis"

SQF_DIR="addon"
SQF_WARN=0

check_sqf() {
    local file="$1"
    local name issues=()

    name="$(basename "$file")"

    # Trailing whitespace
    if grep -n ' $' "$file" | grep -q .; then
        issues+=("trailing whitespace")
    fi

    # Leading tabs (SQF style is spaces)
    if grep -Pn '^\t' "$file" 2>/dev/null | grep -q .; then
        issues+=("leading tab (prefer spaces)")
    fi

    # Rough brace balance (catches obvious forgotten })
    local o c
    o=$(grep -o '{' "$file" | wc -l)
    c=$(grep -o '}' "$file" | wc -l)
    if [[ "$o" -ne "$c" ]]; then
        issues+=("unbalanced braces (open=$o close=$c)")
    fi

    # Missing semicolon: params/private on a single line without closing ;
    # (multi-line params are fine — the ; lands on the ]; line)
    if grep -En '^(params|private)\s+\[.*\]$' "$file" | grep -qv ';$'; then
        issues+=("possible missing semicolon after single-line params/private")
    fi

    if [[ ${#issues[@]} -gt 0 ]]; then
        echo "  ${YEL}⚠ WARN${RST}  ${name}"
        for i in "${issues[@]}"; do echo "           ↳ $i"; done
        SQF_WARN=$((SQF_WARN + 1))
    else
        echo "  ${PASS}  ${name}"
    fi
}

if [[ -d "$SQF_DIR" ]]; then
    while IFS= read -r -d '' f; do
        check_sqf "$f"
    done < <(find "$SQF_DIR" -name "*.sqf" -print0 | sort -z)

    if [[ $SQF_WARN -gt 0 ]]; then
        warn "$SQF_WARN SQF file(s) with style issues (non-blocking)"
    else
        ok "All SQF files clean"
    fi
else
    skip "SQF dir not found ($SQF_DIR)"
fi

# ── 6. Security audit ────────────────────────────────────────────────────────
banner "cargo audit — vulnerability scan"
if command -v cargo-audit &>/dev/null; then
    if run cargo audit; then
        ok "No known vulnerabilities"
    else
        fail "cargo audit found advisories — review before release"
    fi
else
    skip "cargo-audit not installed  →  cargo install cargo-audit"
fi

# ── 7. Unused dependencies ───────────────────────────────────────────────────
banner "cargo machete — unused dependencies"
if command -v cargo-machete &>/dev/null; then
    if run cargo machete; then
        ok "No unused dependencies"
    else
        fail "Unused dependencies found"
    fi
else
    skip "cargo-machete not installed  →  cargo install cargo-machete"
fi

# ── Summary ──────────────────────────────────────────────────────────────────
ELAPSED=$(( $(date +%s) - START_TIME ))
echo
echo "══════════════════════════════════════════════"
printf "  Elapsed : %ds\n" "$ELAPSED"
printf "  Skipped : %d check(s) (optional tools not installed)\n" "$SKIPPED"
echo "──────────────────────────────────────────────"
if [[ $ERRORS -eq 0 ]]; then
    echo "  ${GRN}${BLD}All checks passed. ✔${RST}"
    echo "══════════════════════════════════════════════"
    exit 0
else
    echo "  ${RED}${BLD}${ERRORS} check(s) failed. ✘${RST}"
    echo "══════════════════════════════════════════════"
    exit 1
fi
