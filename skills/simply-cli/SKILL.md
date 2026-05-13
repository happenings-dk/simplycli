---
name: simply-cli
description: Use when operating Simply.com through the local `simply` CLI, including listing Simply products/domains, checking domain availability, buying/registering domains, transferring domains, ordering DNS service, reading or changing nameservers, editing DNS records, DDNS/DynDNS updates, invoices, mail accounts, DNSSEC, or managing multiple Simply account profiles.
---

# Simply CLI

Use the installed `simply` command to operate Simply.com. The CLI stores credentials in `~/.config/simply/config.json`; do not ask the user to paste API keys unless login/setup is explicitly needed, and never print secrets.

## First Checks

Run these before making changes:

```sh
simply accounts
simply products
```

Use a non-default saved profile with:

```sh
simply --profile <name> products
```

Manage profiles:

```sh
simply login --name <profile> --account <S-account-number>
simply accounts
simply use <profile>
simply logout --name <profile>
```

## Common Commands

Domains:

```sh
simply domains check example.com
simply domains register example.com
simply domains buy example.com
simply domains transfer example.com --authid AUTHCODE
simply domains transfer example.com --authid AUTHCODE --keep-nameservers
simply domains dns-service example.com
```

Nameservers:

```sh
simply nameservers get example.com
simply nameservers set example.com --nameserver ns1.example.net --nameserver ns2.example.net
```

DNS records:

```sh
simply dns records example.com
simply dns add-record example.com --type a --name www --data 203.0.113.10 --ttl 3600
simply dns update-record example.com 123 --type txt --name _verify --data "token" --ttl 3600
simply dns delete-record example.com 123
simply dns reload example.com
```

Other API areas:

```sh
simply billing invoices
simply status messages
simply registry dnssec get example.com
simply mail add-account example.com --username hello --password '<password>'
simply ddns --hostname home.example.com
```

## Safety Rules

- Treat domain registration, transfer, DNS service ordering, nameserver changes, DNSSEC deletion, and DNS record deletion/update as high-impact operations.
- Do not pass `--yes` unless the user explicitly requests automation or has already confirmed the exact domain/action.
- For nameserver changes, read current nameservers first with `simply nameservers get <domain>`.
- For domain purchase/registration, check availability first with `simply domains check <domain>`.
- For transfer, make sure the user provided an auth code and knows whether nameservers should change; use `--keep-nameservers` only when they ask to preserve current nameservers.
- After changes, verify with the relevant read command and summarize the actual result.

## Troubleshooting

- If authentication fails, run `simply accounts` and check the selected profile. Use `simply use <profile>` or `--profile <profile>`.
- If network access is blocked by the agent sandbox, rerun the same `simply ...` command with escalated/network permission.
- The CLI lives in `/Users/rasmusjensing/.cargo/bin/simply`; if it is not on PATH, call that absolute path.
