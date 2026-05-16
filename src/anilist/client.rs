use reqwest::header::HeaderMap;
use reqwest::{Client, Error, Response};
use serde::Serialize;

pub struct AnilistClient {
    client: Client,
    headers: HeaderMap,
}

impl AnilistClient {
    const BASE_URL: &'static str = "https://graphql.anilist.co";

    pub fn new(access_token: &str) -> Self {
        let client = Client::new();
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", access_token).parse().unwrap(),
        );
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert("Accept", "application/json".parse().unwrap());

        Self { client, headers }
    }

    pub async fn post<T: Serialize + ?Sized>(&self, body: &T) -> Result<Response, Error> {
        self.client
            .post(Self::BASE_URL)
            .json(body)
            .headers(self.headers.clone())
            .send()
            .await
    }
}
