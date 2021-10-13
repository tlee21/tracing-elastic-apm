use std::{
    fmt::{Display, Formatter, Result},
    ops::Deref,
    sync::{Arc, Mutex},
};

use anyhow::Result as AnyResult;
use reqwest::{header, Client};
use serde_json::{json, Value};
use std::io::Read;
use tokio::time::{sleep, Duration};
use tracing::*;

use crate::config::Authorization;

pub(crate) struct Batch {
    metadata: Value,
    transaction: Option<Value>,
    span: Option<Value>,
    error: Option<Value>,
}

impl Display for Batch {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "{}", json!({ "metadata": self.metadata }))?;

        if let Some(transaction) = &self.transaction {
            writeln!(f, "{}", json!({ "transaction": transaction }))?;
        }

        if let Some(span) = &self.span {
            writeln!(f, "{}", json!({ "span": span }))?;
        }

        if let Some(error) = &self.error {
            writeln!(f, "{}", json!({ "error": error }))?;
        }

        Ok(())
    }
}

impl Batch {
    pub fn new(
        metadata: Value,
        transaction: Option<Value>,
        span: Option<Value>,
        error: Option<Value>,
    ) -> Self {
        Batch {
            metadata,
            transaction,
            span,
            error,
        }
    }
}

pub(crate) struct ApmClient {
    buffer: Arc<Mutex<Vec<Batch>>>,
}

impl ApmClient {
    pub fn new(
        apm_address: String,
        authorization: Option<Authorization>,
        allow_invalid_certs: bool,
        root_cert_path: Option<String>,
        sleep_time: Duration,
    ) -> AnyResult<Self> {
        let authorization = authorization.map(|authorization| match authorization {
            Authorization::SecretToken(token) => format!("Bearer {}", token),
            Authorization::ApiKey(key) => {
                format!(
                    "ApiKey {}",
                    base64::encode(format!("{}:{}", key.id, key.key))
                )
            }
        });

        let mut client_builder = reqwest::ClientBuilder::new();
        if allow_invalid_certs {
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }
        if let Some(path) = root_cert_path {
            let mut buff = Vec::new();
            std::fs::File::open(&path)?.read_to_end(&mut buff)?;
            let cert = reqwest::Certificate::from_pem(&buff)?;
            client_builder = client_builder.add_root_certificate(cert);
        }

        let client = client_builder.build()?;

        let buffer = Arc::new(Mutex::new(Vec::new()));

        let buf = Arc::clone(&buffer);
        tokio::spawn(start_processor(
            apm_address,
            authorization,
            client,
            buf,
            sleep_time,
        ));

        Ok(ApmClient { buffer })
    }

    pub fn send_batch(&self, batch: Batch) {
        let mut buf = self.buffer.lock().unwrap();
        (*buf).push(batch);
    }
}

async fn start_processor(
    apm_address: String,
    authorization: Option<String>,
    client: Client,
    buffer: Arc<Mutex<Vec<Batch>>>,
    sleep_time: Duration,
) {
    loop {
        let batches_to_process: Vec<_> = {
            let mut buf = buffer.lock().unwrap();
            (*buf).drain(..).collect()
        };

        if batches_to_process.is_empty() {
            sleep(sleep_time).await;
            continue;
        }

        for batch in batches_to_process.iter() {
            let mut request = client
                .post(&format!("{}/intake/v2/events", apm_address))
                .header(
                    header::CONTENT_TYPE,
                    header::HeaderValue::from_static("application/x-ndjson"),
                )
                .body(batch.to_string());

            if let Some(authorization) = authorization.clone() {
                request = request.header(header::AUTHORIZATION, authorization);
            }

            let result = request.send().await;
            if let Err(error) = result {
                error!(error = %error, "Error sending batch to APM!");
            }
        }
    }
}
