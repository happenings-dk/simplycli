# Security

This CLI can change DNS, nameservers, and domain orders. Treat API keys as sensitive credentials.

## Reporting

If you find a security issue, report it privately to the maintainer rather than opening a public issue with exploit details or secrets.

## Secret Handling

- Never commit `~/.config/simply/config.json`, `.env`, shell history containing API keys, or screenshots/logs that show credentials.
- If a Simply API key is exposed, revoke or rotate it in the Simply.com control panel.
- Use `simply logout --all` to remove saved local profiles.

## High-Impact Operations

Domain registration, transfer, DNS service ordering, nameserver changes, DNSSEC deletion, and DNS record deletion/update can affect production domains. The CLI prompts for confirmation unless `--yes` is passed.
