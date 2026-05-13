# Contributing

Thanks for helping improve `simply`.

## Development

```sh
cargo fmt
cargo test --bin simply
cargo clippy --bin simply --all-features -- -D warnings
```

The binary is installed locally with:

```sh
cargo install --path . --force
```

To test the install script locally:

```sh
SIMPLYCLI_REPO_URL="$(pwd)" SIMPLYCLI_RAW_BASE="file://$(pwd)" bash scripts/install.sh --skills-only
```

## Credentials

Do not commit real Simply.com credentials. The CLI reads saved credentials from `~/.config/simply/config.json`, or from `SIMPLY_ACCOUNT` and `SIMPLY_API_KEY`.

For testing command parsing without touching the real API, prefer help commands such as:

```sh
cargo run --bin simply -- domains --help
```

## Pull Requests

Keep changes focused. For API behavior changes, include the relevant Simply API endpoint in the PR description and update `docs/usage.md` when the user-facing command surface changes.
