# Usage

`simply` is a command line client for the Simply.com API.

## Authentication

Save credentials for the default profile:

```sh
simply login --account S123456
```

Save multiple profiles:

```sh
simply login --name personal --account S123456
simply login --name work --account S654321
simply accounts
simply use work
simply --profile personal products
```

Credential precedence:

1. `--account` / `--api-key`
2. `SIMPLY_ACCOUNT` / `SIMPLY_API_KEY`
3. Saved profile in `~/.config/simply/config.json`

## Products

```sh
simply products
```

## Domains

```sh
simply domains check example.com
simply search spendless
simply domains search spendless --all
simply domains register example.com
simply domains buy example.com --yes
simply domains transfer example.com --authid AUTHCODE
simply domains transfer example.com --authid AUTHCODE --keep-nameservers
simply domains dns-service example.com
```

Domain registration, transfer, and DNS service ordering require confirmation unless `--yes` is passed.

## Nameservers

```sh
simply nameservers get example.com
simply nameservers set example.com \
  --nameserver ns1.simply.com \
  --nameserver ns2.simply.com
```

Read current nameservers before changing them. Nameserver updates require confirmation unless `--yes` is passed.

## DNS Records

```sh
simply dns records example.com
simply dns add-record example.com --type a --name home --data 203.0.113.10 --ttl 3600
simply dns update-record example.com 123 --type txt --name _verification --data "token" --ttl 3600
simply dns delete-record example.com 123
simply dns reload example.com
```

## Registry and DNSSEC

```sh
simply registry nameservers get example.com
simply registry dnssec get example.com
simply registry dnssec add example.com --type ds --data "12345 13 2 ..."
simply registry dnssec remove example.com
```

## Mail

```sh
simply mail add-account example.com --username hello --password '<password>'
```

## Billing and Status

```sh
simply billing invoices
simply status messages
```

## DDNS

```sh
simply ddns --hostname home.example.com
simply dyndns --hostname home.example.com --myip 203.0.113.10
```
