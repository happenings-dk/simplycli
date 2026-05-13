use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "simply")]
#[command(about = "Command line client for the Simply.com API")]
#[command(version)]
pub struct Cli {
    /// Simply.com account number, for example S123456.
    #[arg(long, env = "SIMPLY_ACCOUNT", global = true)]
    pub account: Option<String>,

    /// API key for the Simply.com account.
    #[arg(long, env = "SIMPLY_API_KEY", hide_env_values = true, global = true)]
    pub api_key: Option<String>,

    /// Saved account name to use.
    #[arg(long = "profile", env = "SIMPLY_PROFILE", global = true)]
    pub profile: Option<String>,

    /// API base URL.
    #[arg(
        long,
        env = "SIMPLY_BASE_URL",
        default_value = "https://api.simply.com",
        global = true
    )]
    pub base_url: String,

    /// Emit machine-readable JSON. Text endpoints are emitted as JSON strings.
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Save credentials in the local Simply config.
    Login(LoginArgs),
    /// List saved account profiles.
    Accounts,
    /// Switch the default saved account profile.
    Use(UseArgs),
    /// Remove a saved account profile.
    Logout(LogoutArgs),
    /// List account products.
    Products,
    /// Search a bare name across common TLDs and show available domains.
    #[command(alias = "available")]
    Search(SearchDomainsArgs),
    /// Manage DNS zones and records.
    Dns {
        #[command(subcommand)]
        command: DnsCommand,
    },
    /// Manage registry-level domain data.
    Registry {
        #[command(subcommand)]
        command: RegistryCommand,
    },
    /// Get or set registry nameservers.
    Nameservers {
        #[command(subcommand)]
        command: NameserverCommand,
    },
    /// Manage mail accounts.
    Mail {
        #[command(subcommand)]
        command: MailCommand,
    },
    /// Check domain availability.
    Domains {
        #[command(subcommand)]
        command: DomainsCommand,
    },
    /// Read invoices.
    Billing {
        #[command(subcommand)]
        command: BillingCommand,
    },
    /// Read Simply.com server-status messages.
    Status {
        #[command(subcommand)]
        command: StatusCommand,
    },
    /// Place orders.
    Order {
        #[command(subcommand)]
        command: OrderCommand,
    },
    /// User-friendly Dynamic DNS helper.
    Ddns(DdnsArgs),
    /// DynDNS protocol-compatible update endpoint.
    Dyndns(DyndnsArgs),
}

#[derive(Debug, Args)]
pub struct LoginArgs {
    /// Saved profile name.
    #[arg(long, default_value = "default")]
    pub name: String,

    /// Simply.com account number, for example S123456.
    #[arg(long, env = "SIMPLY_ACCOUNT")]
    pub account: String,

    /// API key for the Simply.com account. If omitted, it is read from a hidden prompt.
    #[arg(long, env = "SIMPLY_API_KEY", hide_env_values = true)]
    pub api_key: Option<String>,

    /// Save the account without switching the current profile.
    #[arg(long)]
    pub no_switch: bool,
}

#[derive(Debug, Args)]
pub struct UseArgs {
    /// Saved profile name to make current.
    pub name: String,
}

#[derive(Debug, Args)]
pub struct LogoutArgs {
    /// Saved profile name to remove. Defaults to the current profile.
    #[arg(long)]
    pub name: Option<String>,

    /// Remove all saved profiles.
    #[arg(long)]
    pub all: bool,
}

#[derive(Debug, Subcommand)]
pub enum DnsCommand {
    /// Retrieve DNS zone metadata for a product.
    Zone(ProductArgs),
    /// List DNS records for a product.
    Records(ProductArgs),
    /// Add a DNS record.
    AddRecord(DnsRecordArgs),
    /// Update a DNS record. All fields are sent as a full update.
    UpdateRecord(UpdateDnsRecordArgs),
    /// Delete a DNS record by id.
    DeleteRecord(RecordIdArgs),
    /// Force-reload a DNS zone.
    Reload(ProductArgs),
}

#[derive(Debug, Subcommand)]
pub enum RegistryCommand {
    /// Get or set registry nameservers.
    Nameservers {
        #[command(subcommand)]
        command: NameserverCommand,
    },
    /// Get, add, or remove registry DNSSEC keys.
    Dnssec {
        #[command(subcommand)]
        command: DnssecCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum NameserverCommand {
    /// Retrieve current registry nameservers.
    Get(ProductArgs),
    /// Replace registry nameservers.
    Set(SetNameserversArgs),
}

#[derive(Debug, Subcommand)]
pub enum DnssecCommand {
    /// Retrieve DNSSEC keys published at the registry.
    Get(ProductArgs),
    /// Add a DS or DNSKEY record at the registry.
    Add(AddDnssecArgs),
    /// Remove all DNSSEC keys from a domain.
    Remove(ProductArgs),
}

#[derive(Debug, Subcommand)]
pub enum MailCommand {
    /// Add a mail account to a product.
    AddAccount(AddMailAccountArgs),
}

#[derive(Debug, Subcommand)]
pub enum DomainsCommand {
    /// Check whether a domain can be registered or transferred.
    Check(DomainArgs),
    /// Search a bare name across common TLDs and show available domains.
    Search(SearchDomainsArgs),
    /// Register/buy a new domain with a DNS service.
    #[command(alias = "buy")]
    Register(RegisterDomainArgs),
    /// Transfer a domain to Simply.com.
    Transfer(TransferDomainArgs),
    /// Order a DNS service for an existing domain without registration or transfer.
    DnsService(DnsServiceDomainArgs),
}

#[derive(Debug, Subcommand)]
pub enum BillingCommand {
    /// List paid, credited, and refunded invoices.
    Invoices,
}

#[derive(Debug, Subcommand)]
pub enum StatusCommand {
    /// List current and past server-status messages.
    Messages,
}

#[derive(Debug, Subcommand)]
pub enum OrderCommand {
    /// Order a DNS service, optionally with registration or transfer.
    DnsService(OrderDnsServiceArgs),
}

#[derive(Debug, Args)]
pub struct ProductArgs {
    /// Product object/handle, commonly the domain name.
    pub object: String,
}

#[derive(Debug, Args)]
pub struct DomainArgs {
    /// Fully qualified domain name.
    pub domain: String,
}

#[derive(Debug, Args)]
pub struct SearchDomainsArgs {
    /// Bare domain name, for example "spendless".
    pub name: String,
    /// TLD to check. Repeat to override the default TLD list.
    #[arg(long = "tld", action = ArgAction::Append)]
    pub tlds: Vec<String>,
    /// Show taken/error results too.
    #[arg(long)]
    pub all: bool,
}

#[derive(Debug, Args)]
pub struct RecordIdArgs {
    /// Product object/handle, commonly the domain name.
    pub object: String,
    /// DNS record id.
    pub record_id: u64,
}

#[derive(Debug, Args)]
pub struct DnsRecordArgs {
    /// Product object/handle, commonly the domain name.
    pub object: String,
    /// DNS record type.
    #[arg(long = "type")]
    pub record_type: DnsRecordType,
    /// Hostname or label.
    #[arg(long)]
    pub name: String,
    /// Record data.
    #[arg(long)]
    pub data: String,
    /// TTL in seconds.
    #[arg(long)]
    pub ttl: Option<u32>,
    /// Priority, required for MX and SRV records.
    #[arg(long)]
    pub priority: Option<u32>,
    /// Optional record comment.
    #[arg(long)]
    pub comment: Option<String>,
}

#[derive(Debug, Args)]
pub struct UpdateDnsRecordArgs {
    /// Product object/handle, commonly the domain name.
    pub object: String,
    /// DNS record id.
    pub record_id: u64,
    /// DNS record type.
    #[arg(long = "type")]
    pub record_type: DnsRecordType,
    /// Hostname or label.
    #[arg(long)]
    pub name: String,
    /// Record data.
    #[arg(long)]
    pub data: String,
    /// TTL in seconds.
    #[arg(long)]
    pub ttl: Option<u32>,
    /// Priority, required for MX and SRV records.
    #[arg(long)]
    pub priority: Option<u32>,
    /// Optional record comment.
    #[arg(long)]
    pub comment: Option<String>,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum DnsRecordType {
    A,
    Cname,
    Aaaa,
    Alias,
    Mx,
    Txt,
    Ns,
    Ds,
    Dnskey,
    Srv,
    Spf,
    Soa,
    Https,
    Loc,
    Caa,
    Sshfp,
    Tlsa,
}

impl DnsRecordType {
    pub fn as_api_value(&self) -> &'static str {
        match self {
            Self::A => "A",
            Self::Cname => "CNAME",
            Self::Aaaa => "AAAA",
            Self::Alias => "ALIAS",
            Self::Mx => "MX",
            Self::Txt => "TXT",
            Self::Ns => "NS",
            Self::Ds => "DS",
            Self::Dnskey => "DNSKEY",
            Self::Srv => "SRV",
            Self::Spf => "SPF",
            Self::Soa => "SOA",
            Self::Https => "HTTPS",
            Self::Loc => "LOC",
            Self::Caa => "CAA",
            Self::Sshfp => "SSHFP",
            Self::Tlsa => "TLSA",
        }
    }
}

#[derive(Debug, Args)]
pub struct SetNameserversArgs {
    /// Product object/handle, commonly the domain name.
    pub object: String,
    /// Nameserver hostname. Repeat this flag to pass 2 to 13 nameservers.
    #[arg(long = "nameserver", required = true, action = ArgAction::Append)]
    pub nameservers: Vec<String>,
    /// Skip the confirmation prompt.
    #[arg(long)]
    pub yes: bool,
}

#[derive(Debug, Args)]
pub struct AddDnssecArgs {
    /// Product object/handle, commonly the domain name.
    pub object: String,
    /// DNSSEC key type.
    #[arg(long = "type")]
    pub key_type: DnssecKeyType,
    /// Zone-file formatted DS or DNSKEY record data.
    #[arg(long)]
    pub data: String,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum DnssecKeyType {
    Ds,
    Dnskey,
}

impl DnssecKeyType {
    pub fn as_api_value(&self) -> &'static str {
        match self {
            Self::Ds => "ds",
            Self::Dnskey => "dnskey",
        }
    }
}

#[derive(Debug, Args)]
pub struct AddMailAccountArgs {
    /// Product object/handle, commonly the domain name.
    pub object: String,
    /// Local part of the email address, before @.
    #[arg(long)]
    pub username: String,
    /// Password for the mail account.
    #[arg(long, hide_env_values = true)]
    pub password: String,
}

#[derive(Debug, Args)]
pub struct OrderDnsServiceArgs {
    /// Fully qualified domain name.
    pub domain: String,
    /// Domain action. Must match availability status.
    #[arg(long)]
    pub domain_action: Option<DomainAction>,
    /// Authorization code for transfer actions.
    #[arg(long)]
    pub authid: Option<String>,
    /// Coupon code.
    #[arg(long)]
    pub coupon: Option<String>,
    /// Enable automatic renewal.
    #[arg(long)]
    pub autorenew: Option<bool>,
    /// Skip the confirmation prompt.
    #[arg(long)]
    pub yes: bool,
}

#[derive(Debug, Args)]
pub struct RegisterDomainArgs {
    /// Fully qualified domain name to register.
    pub domain: String,
    /// Coupon code.
    #[arg(long)]
    pub coupon: Option<String>,
    /// Enable automatic renewal.
    #[arg(long)]
    pub autorenew: Option<bool>,
    /// Skip the confirmation prompt.
    #[arg(long)]
    pub yes: bool,
}

#[derive(Debug, Args)]
pub struct TransferDomainArgs {
    /// Fully qualified domain name to transfer.
    pub domain: String,
    /// Authorization code for the domain transfer.
    #[arg(long)]
    pub authid: String,
    /// Transfer without updating nameservers to Simply.com.
    #[arg(long)]
    pub keep_nameservers: bool,
    /// Coupon code.
    #[arg(long)]
    pub coupon: Option<String>,
    /// Enable automatic renewal.
    #[arg(long)]
    pub autorenew: Option<bool>,
    /// Skip the confirmation prompt.
    #[arg(long)]
    pub yes: bool,
}

#[derive(Debug, Args)]
pub struct DnsServiceDomainArgs {
    /// Fully qualified domain name for the DNS service.
    pub domain: String,
    /// Coupon code.
    #[arg(long)]
    pub coupon: Option<String>,
    /// Enable automatic renewal.
    #[arg(long)]
    pub autorenew: Option<bool>,
    /// Skip the confirmation prompt.
    #[arg(long)]
    pub yes: bool,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum DomainAction {
    Register,
    Transfer,
    Transferonly,
    None,
}

impl DomainAction {
    pub fn as_api_value(&self) -> &'static str {
        match self {
            Self::Register => "register",
            Self::Transfer => "transfer",
            Self::Transferonly => "transferonly",
            Self::None => "none",
        }
    }
}

#[derive(Debug, Args)]
pub struct DdnsArgs {
    /// Fully qualified hostname to create or update.
    #[arg(long)]
    pub hostname: String,
    /// Domain to change records on.
    #[arg(long)]
    pub domain: Option<String>,
    /// DNS record name. Overrides the name inferred from hostname.
    #[arg(long)]
    pub record: Option<String>,
    /// IP address to set. If omitted, Simply uses the request client IP.
    #[arg(long)]
    pub myip: Option<String>,
    /// TTL in seconds.
    #[arg(long)]
    pub ttl: Option<u32>,
}

#[derive(Debug, Args)]
pub struct DyndnsArgs {
    /// Fully qualified hostname to create or update.
    #[arg(long)]
    pub hostname: String,
    /// Domain to change records on.
    #[arg(long)]
    pub domain: Option<String>,
    /// IP address to set. If omitted, Simply uses the request client IP.
    #[arg(long)]
    pub myip: Option<String>,
}
