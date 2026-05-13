mod auth;
mod cli;
mod client;
mod config;
mod models;
mod output;

use std::io::{self, Write};

use anyhow::{Result, bail};
use clap::Parser;
use cli::{
    BillingCommand, Cli, Command, DnsCommand, DnsRecordArgs, DnssecCommand, DomainsCommand,
    MailCommand, NameserverCommand, OrderCommand, RegistryCommand, StatusCommand,
    UpdateDnsRecordArgs,
};
use client::SimplyClient;
use models::{
    AddDnssecKeyPayload, AddMailAccountPayload, DnsRecordPayload, OrderDnsServicePayload,
    SetNameserversPayload,
};
use output::{ApiOutput, print_output};

const DEFAULT_DOMAIN_SEARCH_TLDS: &[&str] = &[
    "com", "dk", "io", "app", "co", "net", "org", "dev", "site", "online", "shop", "eu", "ai",
    "money", "finance", "cloud", "software", "tools", "group", "company", "xyz",
];

fn main() -> Result<()> {
    let cli = Cli::parse();
    if let Command::Login(args) = cli.command {
        let api_key = match args.api_key {
            Some(api_key) => api_key,
            None => rpassword::prompt_password("Simply API key: ")?,
        };

        auth::save_credentials(&args.name, &args.account, &api_key, !args.no_switch)?;
        println!(
            "Saved profile `{}` for {} in {}.",
            args.name,
            args.account,
            config::config_path()?.display()
        );
        return Ok(());
    }

    if matches!(cli.command, Command::Accounts) {
        let accounts = config::list_accounts()?;
        if accounts.is_empty() {
            println!("No saved Simply accounts.");
        } else {
            for account in accounts {
                let marker = if account.current { "*" } else { " " };
                println!("{marker} {}\t{}", account.name, account.account);
            }
        }
        return Ok(());
    }

    if let Command::Use(args) = cli.command {
        config::set_current_account(&args.name)?;
        println!("Current Simply profile is now `{}`.", args.name);
        return Ok(());
    }

    if let Command::Logout(args) = cli.command {
        let removed = auth::delete_credentials(args.name.as_deref(), args.all)?;
        if args.all {
            println!("Removed all saved Simply credentials.");
        } else if let Some(name) = removed {
            println!("Removed saved Simply profile `{name}`.");
        } else {
            println!("No matching saved Simply profile found.");
        }
        return Ok(());
    }

    let credentials = auth::load_credentials(cli.profile, cli.account, cli.api_key)?;
    let client = SimplyClient::new(cli.base_url, credentials.account, credentials.api_key)?;
    let output = execute(&client, cli.command)?;

    print_output(output, cli.json)
}

fn execute(client: &SimplyClient, command: Command) -> Result<ApiOutput> {
    match command {
        Command::Login(_) | Command::Accounts | Command::Use(_) | Command::Logout(_) => {
            unreachable!("auth commands are handled before API client creation")
        }
        Command::Products => client.get_json("/2/my/products/"),
        Command::Search(args) => search_domains(client, args.name, args.tlds, args.all),
        Command::Dns { command } => execute_dns(client, command),
        Command::Registry { command } => execute_registry(client, command),
        Command::Nameservers { command } => execute_nameservers(client, command),
        Command::Mail { command } => execute_mail(client, command),
        Command::Domains { command } => execute_domains(client, command),
        Command::Billing { command } => execute_billing(client, command),
        Command::Status { command } => execute_status(client, command),
        Command::Order { command } => execute_order(client, command),
        Command::Ddns(args) => {
            let mut params = vec![("hostname", args.hostname)];
            push_optional(&mut params, "domain", args.domain);
            push_optional(&mut params, "record", args.record);
            push_optional(&mut params, "myip", args.myip);
            push_optional(&mut params, "ttl", args.ttl.map(|ttl| ttl.to_string()));
            client.get_text_query("/2/ddns/", &params)
        }
        Command::Dyndns(args) => {
            let mut params = vec![("hostname", args.hostname)];
            push_optional(&mut params, "domain", args.domain);
            push_optional(&mut params, "myip", args.myip);
            client.get_text_query("/2/dyndns/", &params)
        }
    }
}

fn execute_dns(client: &SimplyClient, command: DnsCommand) -> Result<ApiOutput> {
    match command {
        DnsCommand::Zone(args) => client.get_json(&client.product_path(&args.object, "dns/")),
        DnsCommand::Records(args) => {
            client.get_json(&client.product_path(&args.object, "dns/records/"))
        }
        DnsCommand::AddRecord(args) => {
            let path = client.product_path(&args.object, "dns/records/");
            client.post_json(&path, &dns_record_payload(args))
        }
        DnsCommand::UpdateRecord(args) => {
            let path = client.record_path(&args.object, args.record_id);
            client.put_json(&path, &update_dns_record_payload(args))
        }
        DnsCommand::DeleteRecord(args) => {
            client.delete(&client.record_path(&args.object, args.record_id))
        }
        DnsCommand::Reload(args) => {
            client.post_empty(&client.product_path(&args.object, "dns/reload/"))
        }
    }
}

fn execute_registry(client: &SimplyClient, command: RegistryCommand) -> Result<ApiOutput> {
    match command {
        RegistryCommand::Nameservers { command } => execute_nameservers(client, command),
        RegistryCommand::Dnssec { command } => match command {
            DnssecCommand::Get(args) => {
                client.get_json(&client.product_path(&args.object, "registry/dnssec/"))
            }
            DnssecCommand::Add(args) => {
                let payload = AddDnssecKeyPayload {
                    key_type: args.key_type.as_api_value().to_owned(),
                    data: args.data,
                };
                client.post_json(
                    &client.product_path(&args.object, "registry/dnssec/"),
                    &payload,
                )
            }
            DnssecCommand::Remove(args) => {
                client.delete(&client.product_path(&args.object, "registry/dnssec/"))
            }
        },
    }
}

fn execute_nameservers(client: &SimplyClient, command: NameserverCommand) -> Result<ApiOutput> {
    match command {
        NameserverCommand::Get(args) => {
            client.get_json(&client.product_path(&args.object, "registry/nameservers/"))
        }
        NameserverCommand::Set(args) => {
            if !(2..=13).contains(&args.nameservers.len()) {
                bail!("registry nameserver updates require 2 to 13 --nameserver values");
            }

            confirm_dangerous(
                args.yes,
                &format!(
                    "Replace registry nameservers for {} with: {}",
                    args.object,
                    args.nameservers.join(", ")
                ),
                &args.object,
            )?;

            let payload = SetNameserversPayload {
                nameservers: args.nameservers,
            };
            client.put_json(
                &client.product_path(&args.object, "registry/nameservers/"),
                &payload,
            )
        }
    }
}

fn execute_mail(client: &SimplyClient, command: MailCommand) -> Result<ApiOutput> {
    match command {
        MailCommand::AddAccount(args) => {
            let payload = AddMailAccountPayload {
                username: args.username,
                password: args.password,
            };
            client.post_json(
                &client.product_path(&args.object, "mail/accounts/"),
                &payload,
            )
        }
    }
}

fn execute_domains(client: &SimplyClient, command: DomainsCommand) -> Result<ApiOutput> {
    match command {
        DomainsCommand::Check(args) => client.get_json(&client.domaincheck_path(&args.domain)),
        DomainsCommand::Search(args) => search_domains(client, args.name, args.tlds, args.all),
        DomainsCommand::Register(args) => {
            confirm_dangerous(
                args.yes,
                &format!("Register/buy {} and order DNS service", args.domain),
                &args.domain,
            )?;

            client.post_json(
                "/2/my/order/dnsservice/",
                &OrderDnsServicePayload {
                    domain: args.domain,
                    domainaction: Some("register".to_owned()),
                    authid: None,
                    coupon: args.coupon,
                    autorenew: args.autorenew,
                },
            )
        }
        DomainsCommand::Transfer(args) => {
            let action = if args.keep_nameservers {
                "transferonly"
            } else {
                "transfer"
            };
            confirm_dangerous(
                args.yes,
                &format!("Transfer {} with action `{action}`", args.domain),
                &args.domain,
            )?;

            client.post_json(
                "/2/my/order/dnsservice/",
                &OrderDnsServicePayload {
                    domain: args.domain,
                    domainaction: Some(action.to_owned()),
                    authid: Some(args.authid),
                    coupon: args.coupon,
                    autorenew: args.autorenew,
                },
            )
        }
        DomainsCommand::DnsService(args) => {
            confirm_dangerous(
                args.yes,
                &format!("Order DNS service for existing domain {}", args.domain),
                &args.domain,
            )?;

            client.post_json(
                "/2/my/order/dnsservice/",
                &OrderDnsServicePayload {
                    domain: args.domain,
                    domainaction: Some("none".to_owned()),
                    authid: None,
                    coupon: args.coupon,
                    autorenew: args.autorenew,
                },
            )
        }
    }
}

fn search_domains(
    client: &SimplyClient,
    name: String,
    tlds: Vec<String>,
    include_all: bool,
) -> Result<ApiOutput> {
    let tlds = if tlds.is_empty() {
        DEFAULT_DOMAIN_SEARCH_TLDS
            .iter()
            .map(|tld| (*tld).to_owned())
            .collect()
    } else {
        tlds
    };
    let name = name.trim().trim_end_matches('.').to_owned();
    if name.is_empty() {
        bail!("domain search name cannot be empty");
    }

    let mut results = Vec::new();
    for tld in tlds {
        let tld = tld.trim().trim_start_matches('.').trim_end_matches('.');
        if tld.is_empty() {
            continue;
        }

        let domain = format!("{name}.{tld}");
        let response = client.get_value(&client.domaincheck_path(&domain))?;
        let Some(domain_result) = response.get("domain") else {
            continue;
        };

        let available = domain_result
            .get("available")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        if available || include_all {
            results.push(domain_result.clone());
        }
    }

    Ok(ApiOutput::Json(serde_json::json!({
        "query": name,
        "results": results,
    })))
}

fn execute_billing(client: &SimplyClient, command: BillingCommand) -> Result<ApiOutput> {
    match command {
        BillingCommand::Invoices => client.get_json("/2/my/invoices/"),
    }
}

fn execute_status(client: &SimplyClient, command: StatusCommand) -> Result<ApiOutput> {
    match command {
        StatusCommand::Messages => client.get_json("/2/my/serverstatus/messages/"),
    }
}

fn execute_order(client: &SimplyClient, command: OrderCommand) -> Result<ApiOutput> {
    match command {
        OrderCommand::DnsService(args) => {
            confirm_dangerous(
                args.yes,
                &format!("Order DNS service for {}", args.domain),
                &args.domain,
            )?;

            let payload = OrderDnsServicePayload {
                domain: args.domain,
                domainaction: args
                    .domain_action
                    .map(|domain_action| domain_action.as_api_value().to_owned()),
                authid: args.authid,
                coupon: args.coupon,
                autorenew: args.autorenew,
            };
            client.post_json("/2/my/order/dnsservice/", &payload)
        }
    }
}

fn confirm_dangerous(yes: bool, description: &str, expected: &str) -> Result<()> {
    if yes {
        return Ok(());
    }

    eprintln!("{description}");
    eprint!("Type `{expected}` to confirm: ");
    io::stderr().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if input.trim() != expected {
        bail!("confirmation did not match; cancelled");
    }

    Ok(())
}

fn dns_record_payload(args: DnsRecordArgs) -> DnsRecordPayload {
    DnsRecordPayload {
        record_type: args.record_type.as_api_value().to_owned(),
        name: args.name,
        data: args.data,
        ttl: args.ttl,
        priority: args.priority,
        comment: args.comment,
    }
}

fn update_dns_record_payload(args: UpdateDnsRecordArgs) -> DnsRecordPayload {
    DnsRecordPayload {
        record_type: args.record_type.as_api_value().to_owned(),
        name: args.name,
        data: args.data,
        ttl: args.ttl,
        priority: args.priority,
        comment: args.comment,
    }
}

fn push_optional(
    params: &mut Vec<(&'static str, String)>,
    key: &'static str,
    value: Option<String>,
) {
    if let Some(value) = value {
        params.push((key, value));
    }
}
