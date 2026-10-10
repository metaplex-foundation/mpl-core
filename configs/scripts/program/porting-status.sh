#!/bin/bash
#
# Reports how much of the JS AVA suite (`clients/js/test`) has been ported to
# the program's Mollusk tests (`programs/mpl-core/tests`).
#
# The JS suite is the behavioural specification; see
# `programs/mpl-core/tests/PORTING.md` for the porting methodology. This script
# keeps the porting table honest without hand-maintained counts:
#
#   - every `test(...)`, `test.serial(...)`, `test.skip(...)` declaration in
#     `clients/js/test/**/*.test.ts` is a JS test, keyed `<file> :: <title>`
#     where <file> is relative to `clients/js/test/`;
#   - every `/// JS: <file> :: <title>` doc-comment marker in
#     `programs/mpl-core/tests/**/*.rs` marks that JS test as ported;
#   - the ```excluded fenced block in PORTING.md lists JS tests that are not
#     portable (one `<file> :: <title>`, `<file>` or `<dir>/*` per line).
#
# Usage:
#   ./configs/scripts/program/porting-status.sh [--summary] [--strict]
#
#   --summary   print the counts only, not the pending list
#   --strict    exit 1 when a marker references a JS file or title that no
#               longer exists (use in CI so renamed JS tests are noticed)
#
# Environment (for testing the script itself): PORTING_JS_DIR, PORTING_RS_DIR
# and PORTING_MD override the three locations above.
#
# Only bash, find, grep, sed, awk, sort and mktemp are needed.

set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
cd "$(dirname "$(dirname "$(dirname "${SCRIPT_DIR}")")")"

# Overridable for testing the script against another tree.
JS_DIR="${PORTING_JS_DIR:-clients/js/test}"
RS_DIR="${PORTING_RS_DIR:-programs/mpl-core/tests}"
PORTING_MD="${PORTING_MD:-${RS_DIR}/PORTING.md}"

SUMMARY=0
STRICT=0
for arg in "$@"; do
    case "${arg}" in
        --summary) SUMMARY=1 ;;
        --strict) STRICT=1 ;;
        -h | --help)
            sed -n '2,/^$/p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "unknown argument: ${arg}" >&2
            exit 2
            ;;
    esac
done

TMP="$(mktemp -d "${TMPDIR:-/tmp}/porting-status.XXXXXX")"
trap 'rm -rf "${TMP}"' EXIT

# --- 1. JS tests: "<file> :: <title>\t<kind>" ------------------------------
#
# Titles are the quoted string ('...', "..." or `...`) that follows the
# declaration on the same line, or on the next line when the declaration is
# `test.serial(` alone (prettier wraps long titles that way). Template-literal
# titles keep their `${...}` placeholders verbatim; the marker must match them
# verbatim too. Aliases such as `const serial = test.serial; serial(...)` are
# not detected; the one file that uses them (helps/fetch.test.ts) is excluded.
: > "${TMP}/js.tsv"
while IFS= read -r file; do
    rel="${file#"${JS_DIR}"/}"
    awk -v file="${rel}" '
        # The quoted string starting at the front of s, with the delimiter
        # removed. Scanning is escape-aware: a backslash-escaped delimiter is
        # part of the title, not its end, so a single-quoted title containing
        # an escaped apostrophe survives whole instead of being truncated at
        # it. The backslash is dropped, so the extracted title reads the way
        # the test name does and the /// JS: marker matches it verbatim. A
        # backslash before anything else is kept as written.
        function unquote(s,    q, i, c, n, out) {
            q = substr(s, 1, 1)
            if (q != "\047" && q != "\"" && q != "`") return ""
            n = length(s)
            out = ""
            for (i = 2; i <= n; i++) {
                c = substr(s, i, 1)
                if (c == "\\" && i < n) {
                    c = substr(s, i + 1, 1)
                    # Drop the backslash only where it escapes a delimiter or
                    # itself; otherwise keep the pair verbatim.
                    if (c == "\047" || c == "\"" || c == "`" || c == "\\") {
                        out = out c
                    } else {
                        out = out "\\" c
                    }
                    i++
                    continue
                }
                if (c == q) return out
                out = out c
            }
            # Unterminated.
            return ""
        }
        {
            line = $0
            if (pending) {
                pending = 0
                sub(/^[[:space:]]+/, "", line)
                title = unquote(line)
                if (title != "") print file " :: " title "\t" kind
                next
            }
            if (match(line, /^[[:space:]]*test(\.[a-z]+)*\(/)) {
                head = substr(line, RSTART, RLENGTH)
                # hooks (test.before, test.afterEach, ...) are not tests
                if (head ~ /\.(before|after)/) next
                kind = (head ~ /\.skip\(/) ? "skip" : "test"
                rest = substr(line, RSTART + RLENGTH)
                sub(/^[[:space:]]+/, "", rest)
                if (rest == "") { pending = 1; next }
                title = unquote(rest)
                if (title != "") print file " :: " title "\t" kind
            }
        }' "${file}" >> "${TMP}/js.tsv"
done < <(find "${JS_DIR}" -name '*.test.ts' | sort)

# --- 2. markers in the Rust tests: "<file> :: <title>" ---------------------
{
    grep -rhE '^[[:space:]]*///[[:space:]]*JS:' --include='*.rs' "${RS_DIR}" || true
} | sed -E 's#^[[:space:]]*///[[:space:]]*JS:[[:space:]]*##; s/[[:space:]]+$//; s/[[:space:]]*::[[:space:]]*/ :: /' \
    > "${TMP}/markers.txt"

# --- 3. excluded entries from PORTING.md -----------------------------------
if [ -f "${PORTING_MD}" ]; then
    awk '
        /^```excluded[[:space:]]*$/ { inblock = 1; next }
        /^```/ { inblock = 0 }
        inblock && NF && $0 !~ /^[[:space:]]*#/ {
            sub(/[[:space:]]+$/, "")
            sub(/^[[:space:]]+/, "")
            print
        }' "${PORTING_MD}" | sed -E 's/[[:space:]]*::[[:space:]]*/ :: /' > "${TMP}/excluded.txt"
else
    : > "${TMP}/excluded.txt"
fi

# --- 4. join -----------------------------------------------------------------
awk -F '\t' -v summary="${SUMMARY}" '
    function file_of(key) { return substr(key, 1, index(key, " :: ") - 1) }
    function is_excluded(key,    f, d) {
        if (key in ex_test) { ex_test_used[key] = 1; return 1 }
        f = file_of(key)
        if (f in ex_file) { ex_file_used[f] = 1; return 1 }
        for (d in ex_dir) {
            if (substr(f, 1, length(d) + 1) == d "/") { ex_dir_used[d] = 1; return 1 }
        }
        return 0
    }
    mode == 1 {
        if (!($1 in js)) { js[$1] = 1; order[++n] = $1 }
        if ($2 == "skip") skipped++
        next
    }
    mode == 2 { if (NF && !($1 in marker)) { marker[$1] = 1; nmarkers++ }; next }
    mode == 3 {
        if (index($1, " :: ") > 0) ex_test[$1] = 1
        else if ($1 ~ /\/\*$/) ex_dir[substr($1, 1, length($1) - 2)] = 1
        else ex_file[$1] = 1
        next
    }
    END {
        for (i = 1; i <= n; i++) {
            key = order[i]
            excluded = is_excluded(key)
            if (key in marker) {
                ported++
                if (excluded) both[++nboth] = key
            } else if (excluded) {
                nexcluded++
            } else {
                pending[++npending] = key
            }
        }
        for (key in marker) if (!(key in js)) stale[++nstale] = key
        for (key in ex_test) if (!(key in js)) stale_ex[++nstale_ex] = key
        for (f in ex_file) if (!(f in ex_file_used)) stale_ex[++nstale_ex] = f
        for (d in ex_dir) if (!(d in ex_dir_used)) stale_ex[++nstale_ex] = d "/*"

        portable = n - nexcluded
        pct = portable > 0 ? 100 * ported / portable : 0
        printf "JS tests:   %d (%d test.skip)\n", n, skipped
        printf "ported:     %d\n", ported
        printf "excluded:   %d\n", nexcluded
        printf "pending:    %d\n", npending
        printf "progress:   %d/%d portable JS tests ported (%.1f%%)\n", ported, portable, pct

        if (!summary && npending > 0) {
            print ""
            print "pending (file :: title):"
            for (i = 1; i <= npending; i++) print "  " pending[i]
        }

        fflush()
        if (nboth > 0) {
            print "" > "/dev/stderr"
            print "warning: excluded in PORTING.md but carrying a marker (counted as ported):" > "/dev/stderr"
            for (i = 1; i <= nboth; i++) print "  " both[i] > "/dev/stderr"
        }
        if (nstale_ex > 0) {
            print "" > "/dev/stderr"
            print "warning: excluded entries in PORTING.md that match no JS test:" > "/dev/stderr"
            for (i = 1; i <= nstale_ex; i++) print "  " stale_ex[i] > "/dev/stderr"
        }
        if (nstale > 0) {
            print "" > "/dev/stderr"
            print "error: markers referencing JS tests that do not exist (renamed or removed?):" > "/dev/stderr"
            for (i = 1; i <= nstale; i++) print "  " stale[i] > "/dev/stderr"
            exit 3
        }
    }' mode=1 "${TMP}/js.tsv" mode=2 "${TMP}/markers.txt" mode=3 "${TMP}/excluded.txt" || {
    status=$?
    if [ "${status}" -eq 3 ]; then
        if [ "${STRICT}" -eq 1 ]; then
            echo "porting-status: stale markers found (--strict)" >&2
            exit 1
        fi
        exit 0
    fi
    exit "${status}"
}
