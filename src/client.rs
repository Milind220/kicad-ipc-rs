use std::time::Duration;

use crate::error::KiCadError;

#[derive(Clone, Debug)]
pub struct KiCadClient {
    config: ClientConfig,
}

#[derive(Clone, Debug)]
struct ClientConfig {
    timeout: Duration,
    socket_path: Option<String>,
    token: Option<String>,
    client_name: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ClientBuilder {
    config: ClientConfig,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            config: ClientConfig {
                timeout: Duration::from_millis(3_000),
                socket_path: None,
                token: None,
                client_name: None,
            },
        }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    pub fn socket_path(mut self, socket_path: impl Into<String>) -> Self {
        self.config.socket_path = Some(socket_path.into());
        self
    }

    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.config.token = Some(token.into());
        self
    }

    pub fn client_name(mut self, client_name: impl Into<String>) -> Self {
        self.config.client_name = Some(client_name.into());
        self
    }

    pub async fn connect(self) -> Result<KiCadClient, KiCadError> {
        Ok(KiCadClient {
            config: self.config,
        })
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl KiCadClient {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub async fn connect() -> Result<Self, KiCadError> {
        ClientBuilder::new().connect().await
    }

    pub fn timeout(&self) -> Duration {
        self.config.timeout
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::ClientBuilder;

    #[tokio::test]
    async fn builder_overrides_timeout() {
        let timeout = Duration::from_secs(9);
        let client = ClientBuilder::new()
            .timeout(timeout)
            .connect()
            .await
            .expect("builder should connect in baseline scaffold");

        assert_eq!(client.timeout(), timeout);
    }
}
