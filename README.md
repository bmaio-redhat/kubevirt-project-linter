# kubevirt-project-linter

MCP server for static analysis and documentation accuracy in the [kubevirt-ui](https://github.com/kubevirt-ui/kubevirt-ui) Playwright E2E project.

Addresses the "documentation drift" problem: README tables of setup rules, fixture-to-folder mappings, and env variable references go stale over time. This server reads the source of truth directly from TypeScript source files and returns authoritative answers. Also enforces spec file conventions so agents and developers do not need to memorise them.

## Tools

| Tool | Description |
|------|-------------|
| `get_setup_rules` | Parse `project-dependencies/rule-engine/setup-rules.ts` → authoritative setup rule list with IDs, phases, descriptions |
| `get_teardown_rules` | Parse teardown modules → authoritative teardown rule list |
| `get_fixture_map` | For every folder under `playwright/tests/`, return the fixture file it imports — replaces the stale README table |
| `get_env_vars` | Parse `EnvVariables` class and `.env.example` → authoritative table of env var names and defaults |
| `get_allure_suite_map` | Parse `allure-constants.ts` → feature area to Epic/Feature/Story tags for new test annotations |
| `lint_spec_file` | Check a spec file for: raw `page` usage, hardcoded timeouts, missing `ID(CNV-*)` Jira annotations, missing `cleanup.track`, deep import paths |
| `check_api_ui_parity` | Find tier1 UI specs with console-proxy writes that have no mirrored API spec |
| `validate_std_coverage` | Find `ID(CNV-*)` in specs that have no matching entry in `playwright/docs/` STD files |

## Installation

### Prerequisites

- [Rust](https://rustup.rs) 1.75 or later

### Build

```bash
git clone https://github.com/kubevirt-ui/kubevirt-project-linter
cd kubevirt-project-linter
cargo build --release
```

The binary is produced at `target/release/kubevirt-project-linter`.

### Add to Cursor `mcp.json`

In your project's `.cursor/mcp.json`:

```json
"kubevirt-project-linter": {
  "command": "/path/to/kubevirt-project-linter/target/release/kubevirt-project-linter",
  "args": [],
  "env": {
    "KUBEVIRT_PROJECT_ROOT": "/path/to/kubevirt-ui"
  }
}
```

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `KUBEVIRT_PROJECT_ROOT` | `~/Developer/Projects/kubevirt-ui` | Root of the kubevirt-ui repo |
| `LINTER_LOG` | `info` | Log level for stderr (`error`, `warn`, `info`, `debug`) |

## Lint rules reference

`lint_spec_file` checks for the following violations:

| Rule ID | Violation | Fix |
|---------|-----------|-----|
| `no-raw-page` | Direct `page.click()` / `page.fill()` in spec | Use a page object method instead |
| `use-test-timeouts` | Hardcoded numeric timeout (e.g. `timeout: 30000`) | Use `TestTimeouts.*` constants |
| `require-jira-id` | `test.step()` calls with no `ID(CNV-*)` annotation | Add `ID(CNV-XXXXX)` inside `test.step()` |
| `require-cleanup-track` | Resource creation with no `cleanup.track()` | Track all created resources for teardown |
| `use-barrel-imports` | Deep import path into `page-objects/` or `clients/` | Import from barrel index |

## Running tests

```bash
cargo test
```
