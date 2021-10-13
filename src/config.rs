//! Layer configuration.

use tokio::time::Duration;

use crate::model::{Cloud, Framework, Language, Process, Runtime, ServiceNode, System, User};

/// Name for the trace id field, if one needs to be supplied manually.
pub const TRACE_ID_FIELD_NAME: &str = "trace_id";

pub struct Service {
    pub(crate) version: Option<String>,
    pub(crate) environment: Option<String>,
    pub(crate) language: Option<Language>,
    pub(crate) runtime: Option<Runtime>,
    pub(crate) framework: Option<Framework>,
    pub(crate) node: Option<ServiceNode>,
}

impl Service {
    pub fn new(
        version: Option<String>,
        environment: Option<String>,
        language: Option<Language>,
        runtime: Option<Runtime>,
        framework: Option<Framework>,
        node: Option<ServiceNode>,
    ) -> Self {
        Service {
            version,
            environment,
            language,
            runtime,
            framework,
            node,
        }
    }
}

pub struct ApiKey {
    pub(crate) id: String,
    pub(crate) key: String,
}

impl ApiKey {
    pub fn new(id: String, key: String) -> Self {
        ApiKey { id, key }
    }
}

/// APM authorization method.
pub enum Authorization {
    SecretToken(String),
    ApiKey(ApiKey),
}

#[derive(Default)]
pub struct Config {
    pub(crate) apm_address: String,
    pub(crate) authorization: Option<Authorization>,
    pub(crate) service: Option<Service>,
    pub(crate) process: Option<Process>,
    pub(crate) system: Option<System>,
    pub(crate) user: Option<User>,
    pub(crate) cloud: Option<Cloud>,
    pub(crate) allow_invalid_certs: bool,
    pub(crate) root_cert_path: Option<String>,
    pub(crate) sleep_time: Duration,
}

impl Config {
    pub fn new(apm_address: String) -> Self {
        Config {
            apm_address,
            sleep_time: Duration::from_millis(1000),
            ..Default::default()
        }
    }

    pub fn allow_invalid_certificates(mut self, verify: bool) -> Self {
        self.allow_invalid_certs = verify;
        self
    }

    pub fn with_root_cert_path(mut self, cert_path: String) -> Self {
        self.root_cert_path = Some(cert_path);
        self
    }

    pub fn with_authorization(mut self, authorization: Authorization) -> Self {
        self.authorization = Some(authorization);
        self
    }

    pub fn with_service(mut self, service: Service) -> Self {
        self.service = Some(service);
        self
    }

    pub fn with_process(mut self, process: Process) -> Self {
        self.process = Some(process);
        self
    }

    pub fn with_system(mut self, system: System) -> Self {
        self.system = Some(system);
        self
    }

    pub fn with_user(mut self, user: User) -> Self {
        self.user = Some(user);
        self
    }

    pub fn with_cloud(mut self, cloud: Cloud) -> Self {
        self.cloud = Some(cloud);
        self
    }

    pub fn with_sleep_time(mut self, sleep_time: Duration) -> Self {
        self.sleep_time = sleep_time;
        self
    }
}
