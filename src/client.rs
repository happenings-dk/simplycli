use anyhow::{Context, Result, bail};
use reqwest::StatusCode;
use reqwest::blocking::{Client as HttpClient, RequestBuilder, Response};
use serde::Serialize;
use serde_json::Value;
use urlencoding::encode;

use crate::output::ApiOutput;

#[derive(Debug)]
pub struct SimplyClient {
    http: HttpClient,
    base_url: String,
    api_key: String,
}

impl SimplyClient {
    pub fn new(base_url: String, api_key: String) -> Result<Self> {
        if api_key.trim().is_empty() {
            bail!("Simply API key is empty; pass --api-key or set SIMPLY_API_KEY");
        }

        let http = HttpClient::builder()
            .user_agent(concat!("simply/", env!("CARGO_PKG_VERSION")))
            .build()
            .context("failed to build HTTP client")?;

        Ok(Self {
            http,
            base_url: base_url.trim_end_matches('/').to_owned(),
            api_key,
        })
    }

    pub fn get_json(&self, path: &str) -> Result<ApiOutput> {
        self.send(self.auth(self.http.get(self.url(path))))
    }

    pub fn get_value(&self, path: &str) -> Result<Value> {
        match self.get_json(path)? {
            ApiOutput::Json(value) => Ok(value),
            ApiOutput::Text(_) => bail!("expected JSON response from API"),
        }
    }

    pub fn post_json<T>(&self, path: &str, payload: &T) -> Result<ApiOutput>
    where
        T: Serialize + ?Sized,
    {
        self.send(self.auth(self.http.post(self.url(path))).json(payload))
    }

    pub fn post_empty(&self, path: &str) -> Result<ApiOutput> {
        self.send(self.auth(self.http.post(self.url(path))))
    }

    pub fn put_json<T>(&self, path: &str, payload: &T) -> Result<ApiOutput>
    where
        T: Serialize + ?Sized,
    {
        self.send(self.auth(self.http.put(self.url(path))).json(payload))
    }

    pub fn delete(&self, path: &str) -> Result<ApiOutput> {
        self.send(self.auth(self.http.delete(self.url(path))))
    }

    pub fn product_path(&self, object: &str, suffix: &str) -> String {
        format!("/2/my/products/{}/{}", encode(object), suffix)
    }

    pub fn record_path(&self, object: &str, record_id: u64) -> String {
        format!(
            "/2/my/products/{}/dns/records/{}/",
            encode(object),
            record_id
        )
    }

    pub fn domaincheck_path(&self, domain: &str) -> String {
        format!("/2/my/domaincheck/{}/", encode(domain))
    }

    fn auth(&self, request: RequestBuilder) -> RequestBuilder {
        request.bearer_auth(&self.api_key)
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn send(&self, request: RequestBuilder) -> Result<ApiOutput> {
        let response = request.send().context("request failed")?;
        let status = response.status();
        let rate_limit = response
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned);
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned);

        if !status.is_success() {
            return Self::error_response(status, response, rate_limit, retry_after);
        }

        Self::success_response(response)
    }

    fn success_response(response: Response) -> Result<ApiOutput> {
        let is_json = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|content_type| content_type.starts_with("application/json"));

        if is_json {
            let value = response
                .json::<Value>()
                .context("failed to parse JSON response")?;
            Ok(ApiOutput::Json(value))
        } else {
            let text = response.text().context("failed to read text response")?;
            Ok(ApiOutput::Text(text))
        }
    }

    fn error_response(
        status: StatusCode,
        response: Response,
        rate_limit: Option<String>,
        retry_after: Option<String>,
    ) -> Result<ApiOutput> {
        let body = response.text().context("failed to read error response")?;
        let mut message = format!("API request failed with HTTP {status}");

        if let Some(remaining) = rate_limit {
            message.push_str(&format!("; rate limit remaining: {remaining}"));
        }

        if let Some(retry_after) = retry_after {
            message.push_str(&format!("; retry after: {retry_after}"));
        }

        if !body.trim().is_empty() {
            message.push_str(&format!("; response: {}", body.trim()));
        }

        bail!(message)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    use std::thread;

    use super::*;

    #[test]
    fn sends_api_key_as_bearer_token() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut headers = Vec::new();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line.trim().is_empty() {
                    break;
                }
                headers.push(line.trim().to_owned());
            }
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
                )
                .unwrap();
            headers
        });

        let client = SimplyClient::new(base_url, "test-key".to_owned()).unwrap();
        client.get_json("/2/my/products/").unwrap();

        let headers = server.join().unwrap();
        assert_eq!(headers[0], "GET /2/my/products/ HTTP/1.1");
        let authorization = headers.iter().find_map(|header| {
            let (name, value) = header.split_once(':')?;
            name.eq_ignore_ascii_case("authorization")
                .then(|| value.trim().to_owned())
        });
        assert_eq!(authorization.as_deref(), Some("Bearer test-key"));
    }

    #[test]
    fn rejects_empty_api_key() {
        assert!(SimplyClient::new("https://api.simply.com".to_owned(), " ".to_owned()).is_err());
    }
}
