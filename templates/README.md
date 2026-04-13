# cli-forge Templates

This directory is the single source of truth for all templates used by the
cli-forge skill family. No stage directory should contain its own template
copies.

## Directory Structure

```text
templates/
├── scaffold/        Baseline project templates (Stage 3: Scaffold)
├── extensions/      Optional feature templates (Stage 4: Extend)
│   ├── repl.rs.tpl
│   ├── stream.rs.tpl
│   └── daemon/      Daemon reference implementation
├── publish/         Release automation asset pack (Stage 6: Publish)
└── README.md        This file
```

## Template Categories

### Scaffold Templates (`scaffold/`)

Used by the Scaffold stage to create a new Rust CLI Skill project. These
templates contain `{{token}}` placeholders that are expanded using values from
the approved `cli-plan.yml`.

| Template          | Generated File      | Purpose                           |
| ----------------- | ------------------- | --------------------------------- |
| `Cargo.toml.tpl`  | `Cargo.toml`        | Package metadata and dependencies |
| `README.md.tpl`   | `README.md`         | User-facing documentation         |
| `SKILL.md.tpl`    | `SKILL.md`          | Skill contract                    |
| `cli_test.rs.tpl` | `tests/cli_test.rs` | CLI integration tests             |
| `context.rs.tpl`  | `src/context.rs`    | Active Context handling           |
| `help.rs.tpl`     | `src/help.rs`       | Help system module                |
| `lib.rs.tpl`      | `src/lib.rs`        | Library entry point               |
| `main.rs.tpl`     | `src/main.rs`       | Program entry point               |

### Extension Templates (`extensions/`)

Used by the Extend stage to add optional capabilities to an existing
scaffolded project.

| Template        | Generated File  | Feature                         |
| --------------- | --------------- | ------------------------------- |
| `stream.rs.tpl` | `src/stream.rs` | Streaming output                |
| `repl.rs.tpl`   | `src/repl.rs`   | Interactive REPL mode           |
| `daemon/`       | Full crate      | Daemon app server with JSON-RPC |

### Publish Asset Pack (`publish/`)

Copied into the root of a target generated CLI skill repository to provide
release automation. This pack does not contain project templates.

Included assets:

- `package.json` — npm script definitions for release commands
- `.releaserc.json` — semantic-release configuration
- `.github/workflows/release.yml` — GitHub Actions release workflow
- `release/skill-release.config.json` — skill release configuration
- `scripts/install-current-release.sh` — clone-first install helper
- `scripts/release/*.mjs` — release automation scripts

## Rules

1. All stages reference templates from this directory. No stage directory
   should contain its own `.tpl` files.
2. The daemon reference implementation under `extensions/daemon/` is a
   complete Cargo crate that can be built independently.
3. The publish asset pack does not include project templates. Scaffold and
   extension templates live in their respective directories above.
