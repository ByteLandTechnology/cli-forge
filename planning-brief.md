# cli-forge Planning Brief

Use this brief as the authoritative planning-stage contract for any `cli-forge`
workflow. It is the single source of truth for planning decisions across all
stages. Stage-specific supplements exist only in `cli-forge-publish/` and
`cli-forge-distribute/`.

## Purpose And Scope

- Plan Rust-based CLI Skills that will be designed, planned, scaffolded,
  extended, validated, published, or distributed through the `cli-forge` stage
  skills.
- Lock the user-visible CLI contract before implementation starts.
- Surface risks early enough that the workflow can choose scope, sequencing, and
  acceptance criteria deliberately instead of discovering them mid-build.

## Planning Contract

Every plan must explicitly lock these decisions:

- the primary CLI entrypoint and any subcommands
- the canonical shipped invocation using the bare executable name
- the local-development support form using `cargo run -- ...`
- the built-binary verification form using `./target/release/<skill-name> ...`
- required flags, optional flags, defaults, and input sources
- `SKILL.md` contract surfaces that must stay aligned with code and help text
- output formats supported by each command
- runtime-directory and Active Context behavior when those surfaces exist
- daemon lifecycle command surfaces and recovery rules when daemon mode is in
  scope
- which optional capabilities are in scope: base only, `stream`, `repl`,
  `daemon`, or publish/release follow-through
- whether publish-oriented work is about the default repo-native release path,
  the optional npm channel, or both with repo-native release remaining primary

If a plan cannot answer one of those items, it is not ready to move past the
planning stage.

## Approval And Handoff UX

- When a stage needs user approval or needs to hand work to the next stage, use
  a dialog-based chooser (for example, `request_user_input`). This is required
  for every approval and handoff step.
- Offer 2 or 3 explicit options with the recommended path first, such as
  approve and continue, request changes, or stop for now.
- Never require the user to type an exact phrase, a skill name, or the literal
  word `approved` just to continue.
- Never present a numbered menu or ask the user to reply with a digit,
  sequence number, or manually typed option label.
- If dialog tooling is unavailable in the current runtime, stop and report that
  the workflow is waiting for a dialog-capable handoff surface. Do not fall
  back to free-form or numbered text input.

## CLI And Skill Expectations

- The skill remains CLI-first: one executable command is the required
  integration surface.
- The final shipped contract uses the bare executable name:
  `<skill-name> ...`.
- Local development may use `cargo run -- ...`.
- Release verification may use `./target/release/<skill-name> ...`.
- These support forms are for development and release verification only; they
  are not the shipped agent-facing contract.
- `SKILL.md` is the contract that implementation must match. Plans must name
  the required sections and call out any user-visible changes.
- The approved description contract must stay synchronized across
  `Cargo.toml`, `SKILL.md`, `README.md`, and help summaries.
- When daemon behavior is in scope, plans must lock the shared managed
  background contract: `daemon start|stop|restart|status`, default
  single-instance control, terminal outcome or explicit timeout, CLI-only
  recovery, and attached foreground execution out of scope.
- Repository-owned automation and release plumbing must stay outside generated
  skill packages unless a stage explicitly handles publish concerns.
- When publish concerns are in scope, plans must keep repo-native GitHub
  Release publication primary and treat clone-first installation and release
  evidence as repository-owned surfaces.
- Validation should confirm publish-channel context is explicit, but the choice
  between repo-native and npm publication still belongs to the relevant publish
  or distribute child skill.

## Output, Stream, REPL, And Help

Plans must state the intended behavior for:

- default structured output format: YAML unless the plan justifies otherwise
- explicit `--format yaml|json|toml` support
- structured errors on `stderr` with stable machine-readable fields
- plain-text `--help` vs structured `help` behavior
- daemon lifecycle states, timeout semantics, and recovery messaging when
  daemon control is in scope
- `--stream` framing rules if streaming is in scope
- `--repl` interaction model if REPL is in scope

When a capability is not in scope, the plan should say so explicitly instead
of leaving it ambiguous.

## Structure And Testing

Plans should include the expected generated structure and validation evidence:

- root contract files such as `Cargo.toml`, `SKILL.md`, and user-facing docs
- source layout under `src/` and CLI contract tests under `tests/`
- CLI integration tests that invoke the built binary
- coverage for help, format switching, error handling, Active Context, and any
  enabled `stream`, `repl`, or `daemon` behavior
- verification steps such as `cargo build`, `cargo test`,
  `cargo clippy -- -D warnings`, and `cargo fmt --check`

## Risks To Check Explicitly In Every Plan

- drift between `SKILL.md`, help text, and actual CLI behavior
- daemon wording drifting between design, plan, scaffold, extend, and validate
  surfaces
- unclear default output vs explicit `--format` behavior
- unsupported `--stream --format toml` paths when streaming is enabled
- REPL usability vs machine-readable output expectations
- Active Context persistence and override precedence
- accidental leakage of repository-owned automation into generated packages
- release credentials, target artifacts, and destination configuration when
  publish work is in scope
- repo version, git tag, release page, binary assets, and release evidence
  drifting apart when publish work is in scope
- validation reporting failing to distinguish repo-native release checks from
  optional npm package-set checks
- publish-channel confusion where repo-native release and npm publication are
  mixed into one undefined path
