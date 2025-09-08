#!/usr/bin/env bash
set -euo pipefail

# RSB Central Test Orchestrator

BLUE="\033[34;1m"
GREEN="\033[32;1m"
YELLOW="\033[33;1m"
RED="\033[31;1m"
CYAN="\033[36;1m"
GREY="\033[90m"
RESET="\033[0m"

box() {
  local title="$1"; shift
  local border="$(printf '─%.0s' $(seq 1 68))"
  echo -e "${BLUE}┌─ ${title} ${border:0:$((70-${#title}))}┐${RESET}"
  while IFS= read -r line; do
    printf "${BLUE}│${RESET} %s\n" "$line"
  done
  echo -e "${BLUE}└${border:0:72}┘${RESET}"
}

section() { box "$1" <<< "$2"; }

# 1) Generate/refresh test inventory
inv_gen() {
  printf "Generating test inventory...\n" >&2
  {
    echo "# Test Inventory (auto-generated)"
    echo "# Format: <src_path> | stem=<name> | tests:<count> [files]"
    echo
    while IFS= read -r f; do
      stem="$(basename "$f" .rs)"
      matches="$(rg -n --no-heading -S "\\b${stem}\\b" tests || true)"
      count=$(printf "%s" "$matches" | rg -c "^" || true)
      [ -z "$matches" ] && count=0
      files=$(printf "%s" "$matches" | cut -d: -f1 | sort -u | tr '\n' ' ' | sed 's/ *$//')
      echo "$f | stem=$stem | tests:$count ${files:+[$files]}"
    done < <(rg --files src | sort)
  } > test_inventory.log
}

# 2) Run cargo tests and capture output
run_tests() {
  printf "Running cargo tests...\n" >&2
  local out_file="target/test-output.log"
  mkdir -p target
  # Ensure a writable tmp dir for cap_stream tests
  export XDG_TMP="${XDG_TMP:-$(mktemp -d)}"
  if ! cargo test -q | tee "$out_file"; then
    return 1
  fi
}

# 3) Summarize coverage from inventory
coverage_summary() {
  local total covered uncovered
  total=$(rg -c "^src/" test_inventory.log || echo 0)
  covered=$(rg -n "tests:[1-9]" test_inventory.log | wc -l | awk '{print $1}')
  uncovered=$(rg -n "tests:0" test_inventory.log | wc -l | awk '{print $1}')
  echo -e "Total src files: ${CYAN}${total}${RESET}"
  echo -e "Covered (heuristic): ${GREEN}${covered}${RESET}"
  echo -e "Uncovered (heuristic): ${YELLOW}${uncovered}${RESET}"
  echo
  echo "Uncovered list (up to 20):"
  rg -n "tests:0" test_inventory.log | sed -n '1,20p' | sed 's/^\([0-9]*:\)//'
}

# 3b) Macros coverage focus
macros_coverage() {
  echo "Macro files coverage:"
  awk '/^src\/macros\//{print}' test_inventory.log |
    sed 's/^src\/macros\///' |
    awk -F'\|' '{printf "  %-24s %s\n", $1, $3}'
}

# Verbose mode: show per-file test refs
if [ "${VERBOSE:-}" != "" ]; then
  per_file_refs() {
    echo "Per-file test references:"; echo
    while IFS= read -r line; do
      file=$(echo "$line" | awk -F'|' '{print $1}' | xargs)
      stem=$(basename "$file" .rs)
      echo "- $file"
      rg -n --no-heading -S "\\b${stem}\\b" tests || true
      echo
    done < <(rg --files src | sort)
  }
fi

# 4) Summarize failures
fail_summary() {
  local out_file="target/test-output.log"
  if rg -n "failures:" "$out_file" >/dev/null 2>&1; then
    echo -e "${RED}Detected failing tests:${RESET}"
    awk '/failures:/{flag=1;next}/test result:/{flag=0}flag' "$out_file" | sed 's/^/  /'
    echo
    echo "Panic locations:"
    rg -n "panicked at|thread '" "$out_file" | sed 's/^/  /'
  else
    echo -e "${GREEN}All tests passed.${RESET}"
  fi
}

# 5) Missing features log (if exists)
missing_features() {
  if [ -f missing_features.log ]; then
    cat missing_features.log
  else
    echo "None recorded."
  fi
}

# Orchestration
inv_gen
coverage_text=$(coverage_summary)
section "Inventory Summary" "$coverage_text"

macro_text=$(macros_coverage)
section "Macros Coverage" "$macro_text"
if [ "${VERBOSE:-}" != "" ]; then
  v_text=$(per_file_refs)
  section "Verbose: Per-File Test Refs" "$v_text"
fi

set +e
run_tests
rc=$?
set -e

fail_text=$(fail_summary)
section "Cargo Test Summary" "$fail_text"

mf_text=$(missing_features)
section "Missing Features" "$mf_text"

if [ $rc -ne 0 ]; then
  echo -e "${RED}Overall: FAIL${RESET}"
else
  echo -e "${GREEN}Overall: PASS${RESET}"
fi

exit $rc
