# Agent Protocol (SEMMAP-first)

You can browse the repo, but orientation and verification MUST be SEMMAP/Neti-driven. Act without unnecessary permission prompts, but still follow any real tool, sandbox, or approval requirements. Neti and SEMMAP binaries are installed in the system path and can be invoked directly, for example `neti check`, `semmap generate`, `semmap trace`, and `semmap cat`.

## Hard rule: no source before orientation

Before reading implementation source beyond the task-defining docs, you MUST:

1. run `semmap generate` (if not already done for this repo state),
2. read `SEMMAP.md` and cite the specific line(s) you used (Purpose plus the relevant layer entries and hotspots),
3. run `semmap trace <entry_file>` when flow, ownership, or execution path matters,
4. state your **minimal working set** (the 2-5 files you intend to read next, and why).

You may read the task-defining docs first: this prompt. Issue discovery and issue updates must go through the `ishoo` CLI against the canonical store. After orientation, if you read additional files beyond the working set, you MUST justify why SEMMAP and the trace output were insufficient.

## Required evidence per iteration

In any iteration where you plan or change code, include:

- the SEMMAP line(s) you used (Purpose plus relevant layer entries and hotspots)
- the exact `semmap trace ...` command(s) you ran, when applicable
- the exact `neti check` result after changes; consult `neti-report.txt` if output is truncated

`neti check` is the canonical verification command. It already runs the configured verification suite, including `cargo test`, the configured Clippy gate, and Neti scan. Do not treat ad hoc `cargo test` or `cargo clippy` runs as a substitute for `neti check`.

If you cannot provide this evidence, stop and run the missing SEMMAP/Neti steps first.

## Workflow

1. Run `semmap generate` and read `SEMMAP.md`. Use the `ishoo` CLI to inspect issues. You can also run `semmap deps` if you need a dependency graph.
2. Write a short Orientation (Purpose, entrypoint, trace target, hotspots, plan).
3. Use `semmap trace <entry_file>` for flow-dependent work or unclear ownership.
4. Declare a minimal working set, then read only those files (prefer `semmap cat`; use other tools if needed).
5. Make minimal edits that respect SEMMAP boundaries; hotspots = smaller diffs + stronger tests.
6. After the change set is in place, run `neti check` (view `neti-report.txt` in repo root for full output). Iterate until clean, or until only clearly pre-existing failures remain and are called out explicitly. You must resolve all technical debt before moving forward, you are not allowed to say "I didnt break it so im leaving it broken", EVER.
7. If you manually exercise a CLI or user-facing flow, report the exact command, the important output, and the exit code when relevant.

## Issue discipline

Work exclusively from the canonical issue store through the CLI. The storage file is a compressed binary; do not attempt to read or modify it directly.

Use these commands as your primary control surface:

- `ishoo agenda --next` (Your primary source of truth for "What's next")
- `ishoo list --compact` (Query the state of the board)
- `ishoo show <ID>` (Read full details/description of a task)
- `ishoo new "<Title>" --category <CAT> --labels <labels>` (Create new work)
- `ishoo edit <ID> --description "<Text>" --depends-on <ID>` (Refine/Link work)
- `ishoo set <ID> <status>` (Update status: active, backlog, done)
- `ishoo help --all` (See all commands and how to use them)

When refining or closing an issue, your `Resolution` update MUST include:

1. **What changed:** Concrete summary of code changes.
2. **Why:** Architectural justification.
3. **Verification:** The exact commands run and the outcome of `neti check`.
4. **Handoff:** If this enables a blocked issue, mention it.

An issue is only DONE when `neti check` is PASS and the status is updated via `ishoo set <ID> done`.

## Minimal close-out

A compliant final report for code work should usually contain:

1. the issue handled
2. the SEMMAP evidence used
3. the key files changed
4. the exact `neti check` outcome
5. any manual CLI or UX verification that was performed
6. whether issue records were updated
