/// Copied from `solana-rpc-client` crate, modified [HttpSender]
/// to allow for passing in default headers. This is useful for
/// passing auth headers to RPC services like GenesysGo.
use reqwest::header::HeaderMap;
use serde::Deserialize;
use serde_json::{json, Value};
use solana_client::rpc_custom_error as custom_error;
use solana_client::rpc_request::{RpcError, RpcRequest, RpcResponseErrorData};
use solana_client::rpc_response::RpcSimulateTransactionResult;
use solana_client::rpc_sender::{RpcSender, RpcTransportStats};
use {
    async_trait::async_trait,
    log::*,
    reqwest::{
        self,
        header::{self, CONTENT_TYPE, RETRY_AFTER},
        StatusCode,
    },
    std::{
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc, RwLock,
        },
        time::{Duration, Instant},
    },
    tokio::time::sleep,
};

#[derive(Deserialize, Debug)]
pub struct RpcErrorObject {
    pub code: i64,
    pub message: String,
}

pub struct HttpSenderWithHeaders {
    client: Arc<reqwest::Client>,
    url: String,
    request_id: AtomicU64,
    stats: RwLock<RpcTransportStats>,
}

/// Nonblocking [`RpcSender`] over HTTP.
impl HttpSenderWithHeaders {
    /// Create an HTTP RPC sender.
    ///
    /// The URL is an HTTP URL, usually for port 8899, as in
    /// "http://localhost:8899". The sender has a default timeout of 30 seconds.
    pub fn new<U: ToString>(url: U, headers: Option<HeaderMap>) -> Self {
        Self::new_with_timeout(url, Duration::from_secs(30), headers)
    }

    /// Create an HTTP RPC sender.
    ///
    /// The URL is an HTTP URL, usually for port 8899.
    pub fn new_with_timeout<U: ToString>(
        url: U,
        timeout: Duration,
        headers: Option<HeaderMap>,
    ) -> Self {
        let mut default_headers = header::HeaderMap::new();
        default_headers.append(
            header::HeaderName::from_static("solana-client"),
            header::HeaderValue::from_str(
                format!("rust/{}", solana_version::Version::default()).as_str(),
            )
            .unwrap(),
        );
        if let Some(headers) = headers {
            default_headers.extend(headers);
        }

        let client = Arc::new(
            reqwest::Client::builder()
                .default_headers(default_headers)
                .timeout(timeout)
                .pool_idle_timeout(timeout)
                .build()
                .expect("build rpc client"),
        );

        Self {
            client,
            url: url.to_string(),
            request_id: AtomicU64::new(0),
            stats: RwLock::new(RpcTransportStats::default()),
        }
    }
}

struct StatsUpdater<'a> {
    stats: &'a RwLock<RpcTransportStats>,
    request_start_time: Instant,
    rate_limited_time: Duration,
}

impl<'a> StatsUpdater<'a> {
    fn new(stats: &'a RwLock<RpcTransportStats>) -> Self {
        Self {
            stats,
            request_start_time: Instant::now(),
            rate_limited_time: Duration::default(),
        }
    }

    fn add_rate_limited_time(&mut self, duration: Duration) {
        self.rate_limited_time += duration;
    }
}

impl<'a> Drop for StatsUpdater<'a> {
    fn drop(&mut self) {
        let mut stats = self.stats.write().unwrap();
        stats.request_count += 1;
        stats.elapsed_time += Instant::now().duration_since(self.request_start_time);
        stats.rate_limited_time += self.rate_limited_time;
    }
}

pub fn build_request_json(req: &RpcRequest, id: u64, params: Value) -> Value {
    let jsonrpc = "2.0";
    json!({
       "jsonrpc": jsonrpc,
       "id": id,
       "method": format!("{}", req),
       "params": params,
    })
}

#[async_trait]
impl RpcSender for HttpSenderWithHeaders {
    fn get_transport_stats(&self) -> RpcTransportStats {
        self.stats.read().unwrap().clone()
    }

    async fn send(
        &self,
        request: RpcRequest,
        params: serde_json::Value,
    ) -> solana_client::client_error::Result<serde_json::Value> {
        let mut stats_updater = StatsUpdater::new(&self.stats);

        let request_id = self.request_id.fetch_add(1, Ordering::Relaxed);
        let request_json = build_request_json(&request, request_id, params).to_string();

        let mut too_many_requests_retries = 5;
        loop {
            let response = {
                let client = self.client.clone();
                let request_json = request_json.clone();
                client
                    .post(&self.url)
                    .header(CONTENT_TYPE, "application/json")
                    .body(request_json)
                    .send()
                    .await
            }?;

            if !response.status().is_success() {
                if response.status() == StatusCode::TOO_MANY_REQUESTS
                    && too_many_requests_retries > 0
                {
                    let mut duration = Duration::from_millis(500);
                    if let Some(retry_after) = response.headers().get(RETRY_AFTER) {
                        if let Ok(retry_after) = retry_after.to_str() {
                            if let Ok(retry_after) = retry_after.parse::<u64>() {
                                if retry_after < 120 {
                                    duration = Duration::from_secs(retry_after);
                                }
                            }
                        }
                    }

                    too_many_requests_retries -= 1;
                    debug!(
                                "Too many requests: server responded with {:?}, {} retries left, pausing for {:?}",
                                response, too_many_requests_retries, duration
                            );

                    sleep(duration).await;
                    stats_updater.add_rate_limited_time(duration);
                    continue;
                }
                return Err(response.error_for_status().unwrap_err().into());
            }

            let mut json = response.json::<serde_json::Value>().await?;
            if json["error"].is_object() {
                return match serde_json::from_value::<RpcErrorObject>(json["error"].clone()) {
                    Ok(rpc_error_object) => {
                        let data = match rpc_error_object.code {
                                    custom_error::JSON_RPC_SERVER_ERROR_SEND_TRANSACTION_PREFLIGHT_FAILURE => {
                                        match serde_json::from_value::<RpcSimulateTransactionResult>(json["error"]["data"].clone()) {
                                            Ok(data) => RpcResponseErrorData::SendTransactionPreflightFailure(data),
                                            Err(err) => {
                                                debug!("Failed to deserialize RpcSimulateTransactionResult: {:?}", err);
                                                RpcResponseErrorData::Empty
                                            }
                                        }
                                    },
                                    custom_error::JSON_RPC_SERVER_ERROR_NODE_UNHEALTHY => {
                                        match serde_json::from_value::<custom_error::NodeUnhealthyErrorData>(json["error"]["data"].clone()) {
                                            Ok(custom_error::NodeUnhealthyErrorData {num_slots_behind}) => RpcResponseErrorData::NodeUnhealthy {num_slots_behind},
                                            Err(_err) => {
                                                RpcResponseErrorData::Empty
                                            }
                                        }
                                    },
                                    _ => RpcResponseErrorData::Empty
                                };

                        Err(RpcError::RpcResponseError {
                            code: rpc_error_object.code,
                            message: rpc_error_object.message,
                            data,
                        }
                        .into())
                    }
                    Err(err) => Err(RpcError::RpcRequestError(format!(
                        "Failed to deserialize RPC error response: {} [{}]",
                        serde_json::to_string(&json["error"]).unwrap(),
                        err
                    ))
                    .into()),
                };
            }
            return Ok(json["result"].take());
        }
    }

    fn url(&self) -> String {
        self.url.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn http_sender_on_tokio_multi_thread() {
        let http_sender = HttpSenderWithHeaders::new("http://localhost:1234".to_string(), None);
        let _ = http_sender
            .send(RpcRequest::GetVersion, serde_json::Value::Null)
            .await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn http_sender_on_tokio_current_thread() {
        let http_sender = HttpSenderWithHeaders::new("http://localhost:1234".to_string(), None);
        let _ = http_sender
            .send(RpcRequest::GetVersion, serde_json::Value::Null)
            .await;
    }
}
