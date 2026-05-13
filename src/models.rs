use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DnsRecordPayload {
    #[serde(rename = "type")]
    pub record_type: String,
    pub name: String,
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SetNameserversPayload {
    pub nameservers: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AddDnssecKeyPayload {
    #[serde(rename = "type")]
    pub key_type: String,
    pub data: String,
}

#[derive(Debug, Serialize)]
pub struct AddMailAccountPayload {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct OrderDnsServicePayload {
    pub domain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domainaction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coupon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autorenew: Option<bool>,
}
