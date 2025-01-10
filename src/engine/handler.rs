use reqwest::Response;
use std::time::Instant;

use crate::{error::Result, filters::Filtrerer, worker::utils::RwalkResponse};

#[derive(Debug, Clone)]
pub struct ResponseHandler {
    filterer: Filtrerer<RwalkResponse>,
    needs_body: bool,
}

impl ResponseHandler {
    pub fn new(filterer: Filtrerer<RwalkResponse>, needs_body: bool) -> Self {
        Self {
            filterer,
            needs_body,
        }
    }

    pub async fn handle_response(
        &self,
        response: Response,
        task: String,
        start_time: Instant,
    ) -> Result<()> {
        let rwalk_response = RwalkResponse::from_response(response, self.needs_body).await?;

        if self.filterer.all(&rwalk_response) {
            println!("{} ({:?})", task, start_time.elapsed());
        }

        Ok(())
    }
}
