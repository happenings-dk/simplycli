# simply

Rust CLI for the [Simply.com API](https://www.simply.com/en/docs/api/).

## Quick Install

Install `simply` plus the Claude Code and Codex skills:

```sh
curl -fsSL https://raw.githubusercontent.com/happenings-dk/simplycli/main/scripts/install.sh | bash
```

## Features

- Product and invoice listing
- Domain availability checks, registration, transfer, and DNS service ordering
- Registry nameserver reads and updates
- DNS record create, update, delete, and reload
- DNSSEC and mail account operations
- Multiple saved account profiles

## Install

The quick installer installs:

- `simply` into Cargo's binary directory, usually `~/.cargo/bin`
- Claude Code skill: `~/.claude/skills/simply-cli`
- Codex skill: `~/.codex/skills/simply-cli`

Full install:

```sh
curl -fsSL https://raw.githubusercontent.com/happenings-dk/simplycli/main/scripts/install.sh | bash
```

Install only the CLI:

```sh
curl -fsSL https://raw.githubusercontent.com/happenings-dk/simplycli/main/scripts/install.sh | bash -s -- --no-skills
```

Install only the skills:

```sh
curl -fsSL https://raw.githubusercontent.com/happenings-dk/simplycli/main/scripts/install.sh | bash -s -- --skills-only
```

From a local checkout:

```sh
cargo install --path . --force
```

## Setup

Create an API key in the Simply.com control panel, then export it:

```sh
export SIMPLY_API_KEY=your-api-key
```

You can also pass `--api-key` on any command. The API key is sent as a Bearer token and identifies the account, so `--account` / `SIMPLY_ACCOUNT` is no longer needed. It is still accepted, so existing setups keep working.

To save credentials globally for your user:

```sh
simply login
```

To save more than one account, give each one a profile name:

```sh
simply login --name personal
simply login --name work
simply accounts
simply use work
simply --profile personal products
```

This writes `~/.config/simply/config.json` with user-only file permissions. Environment variables and CLI flags still take precedence.

## Examples

```sh
# List products
simply products

# List DNS records for a product/domain
simply dns records example.com

# Add an A record
simply dns add-record example.com --type a --name home --data 203.0.113.10 --ttl 3600

# Update an existing record
simply dns update-record example.com 123 --type txt --name _verification --data "token" --ttl 3600

# Delete a record
simply dns delete-record example.com 123

# Read registry nameservers
simply registry nameservers get example.com

# Replace registry nameservers
simply registry nameservers set example.com \
  --nameserver ns1.simply.com \
  --nameserver ns2.simply.com

# Check domain availability
simply domains check example.com

# Search a bare name across common TLDs
simply search spendless
simply domains search spendless --all

# Register/buy a new domain with DNS service
simply domains register example.com
# `buy` is also accepted as an alias:
simply domains buy example.com --yes

# Transfer a domain to Simply.com
simply domains transfer example.com --authid AUTHCODE

# Order DNS service for an existing domain without registration/transfer
simply domains dns-service example.com

# Read registry nameservers
simply nameservers get example.com

# Replace registry nameservers
simply nameservers set example.com \
  --nameserver ns1.simply.com \
  --nameserver ns2.simply.com

# List invoices
simply billing invoices
```

Commands that can charge a card or disrupt DNS require confirmation unless `--yes` is passed.

JSON API responses are printed as pretty JSON by default. Non-JSON responses print as plain text unless `--json` is passed.

More examples are in [docs/usage.md](docs/usage.md).

## API Coverage

The CLI covers the documented Simply.com API v2 endpoints for products, DNS records, DNS zone reloads, registry nameservers, DNSSEC, mail account creation, domain checks, invoices, server-status messages, and DNS service ordering.

Dynamic DNS is not part of this CLI. Routers and DDNS clients such as ddclient or inadyn should use `https://ddns.simply.com/nic/update`; use `simply dns add-record` and `simply dns update-record` to manage records directly.

## Development

```sh
cargo fmt
cargo test --bin simply
cargo clippy --bin simply --all-features -- -D warnings
```

## License

MIT
