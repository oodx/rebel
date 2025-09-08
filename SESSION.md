# RSB Macro + Test Orchestration Session

Overview
- Goal: fix broken RSB patterns, ensure macros “just work,” and make everything testable with clear visibility.
- Scope: dispatch/pre-dispatch, missing macros, helpers, macro/unit/integration tests, central test harness with friendly visuals.

Implemented Changes
- Dispatch robustness:
  - `src/macros/dispatch.rs`: forward all arguments safely (no hard-coded arity). Same for `pre_dispatch!`.
  - Added `tests/dispatch.rs` validating no-arg help, pre-dispatch args, and dispatch arg forwarding.

- Missing macros implemented:
  - Text/time/random/fs data: `str_line!`, `sleep!`, `rand_range!`, `path_canon!`, `path_split!`, `meta_keys!`.
  - Archive: `zip!(list)`, `zip!(extract)`, `zip!(extract,to:)` to match showcase usage.
  - Prelude exports wired for new macros.

- Helpers added:
  - `random::rand_range_usize(min,max)` and `fs::parse_meta_keys(path, into)`.
  - `context`: `XDG_TMP` now respects env `${XDG_TMP:-$HOME/.cache/tmp}` for testability.
  - `streamable::traits`: `.stream_apply` returns `String` (works for `&str` and `String`).
  - `deps` doctest updated for `rand 0.9` (`distr::Alphanumeric`).

Tests Added (high level)
- Macros
  - text: to_number!/param!/str_in!/str_explode!/str_trim!/str_len!/str_line!
  - time_math: date!/math!/sleep!/benchmark!
  - fs_data: sed_around!/sed_lines!/sed_template!/path_canon!/path_split!/meta_keys!
  - json_dict_random: rand_* macros, rand_range!, gen_dict!, rand_dict!
  - streams_exec: pipe()/grep()/run!/shell!/mock_cmd!
  - control_validation: test!/case!/for_in!/with_lock!, require_*/validate!/export!/src!/file_in!
  - core: args!/get_env!

- OS and Streams
  - os: hostname!/user!/home_dir!/current_dir!, pid_of!/process_exists! (mocked), curl/get/post (mocked) 
  - streams: Stream builders, grep/sed/cut/sort/unique, tee/to_file roundtrips

- XCls smoke
  - xgrep: filter_lines; xfilter: filter_transform; xsed: transform_values

Central Orchestration
- `test.sh` runner:
  - Boxed sections: Inventory Summary, Macros Coverage, Cargo Test Summary, Missing Features.
  - Heuristic coverage mapping to `test_inventory.log`.
  - Honors `XDG_TMP` (creates if missing) for cap_stream tests.
  - Verbose mode (`VERBOSE=1 ./test.sh`) shows per-file test references.

Patterns Used (RSB idioms)
- Dual-dispatch via `pre_dispatch!` then `dispatch!`.
- Stream pipelines via `Stream` and `pipe!/cat!/cmd!` + chainable ops.
- Bash-like macros for ergonomics (`validate!/require_*!/case!/test!`).
- Event/trap pattern: `trap!(on: "COMMAND_ERROR")` with `event!(emit ...)` from `run!` failures.
- Archive adapters for tar/zip with auto-detect pack!/unpack! routing.
- XDG paths/bootstrap and EXIT trap for temp cleanup.

What Remains (future work)
- Uncovered modules by heuristic (from latest inventory; many are indirectly covered):
  - macros: dispatch.rs (covered indirectly via integration), mod.rs, test_helpers.rs.
  - core modules: deps.rs, lib.rs.
  - streamable glue: streamable/mod.rs, streamable/traits.rs (basic contracts exist, consider explicit tests), streams.rs (more ops), time.rs, utils.rs.
  - xcls: mod.rs, xfilter.rs (deeper behaviors), xgrep.rs (more scenarios), xsed.rs (key/namespace transforms).
- test.sh enhancements:
  - Add a final uncovered checklist filtered to macro/core focus.
  - Optional JSON export for CI dashboards.

Runbook
- Quick run: `./test.sh` (boxes + PASS/FAIL)
- Verbose mapping: `VERBOSE=1 ./test.sh`
- Direct cargo: `cargo test`

Commits
- Using `gh commit 'message'` (alias: `git add -A && git commit -m "$*"`).

State
- All current tests pass locally. Missing features are tracked in `missing_features.log` (now empty of new items after this work).
