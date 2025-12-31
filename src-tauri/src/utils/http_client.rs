use reqwest::Client;
use std::time::Duration;

lazy_static::lazy_static! {
    pub static ref HTTP_CLIENT: Client = create_client();
}

fn create_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .user_agent("GameManager/0.1.0")
        .build()
        .expect("Failed to create HTTP client")
}
