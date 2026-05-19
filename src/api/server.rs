use crate::api::routes::create_router;
use crate::domain::traits::{SessionRepository, UserRepository};
use crate::infrastructure::database::Database;
use crate::infrastructure::packager::service::PackagerService;
use crate::infrastructure::session::SessionRepositoryImpl;
use crate::infrastructure::user::UserRepositoryImpl;
use anyhow::Context;
use std::sync::Arc;

pub struct ServerState {
    pub user_repository: Arc<dyn UserRepository>,
    pub session_repository: Arc<dyn SessionRepository>,
    pub client_id: String,
    pub client_secret: String,
    pub packager_service: Arc<PackagerService>,
}

pub struct Server {
    state: Arc<ServerState>,
}

impl Server {
    pub fn new(
        db: Arc<Database>,
        packager_service: Arc<PackagerService>,
        client_id: &str,
        client_secret: &str,
    ) -> Self {
        Self {
            state: Arc::new(ServerState {
                session_repository: Arc::new(SessionRepositoryImpl::new(db.clone())),
                user_repository: Arc::new(UserRepositoryImpl::new(db.clone())),
                client_id: client_id.to_owned(),
                client_secret: client_secret.to_owned(),
                packager_service,
            }),
        }
    }

    pub async fn serve(&self) -> anyhow::Result<()> {
        let app = create_router(self.state.clone());
        let listener = tokio::net::TcpListener::bind("0.0.0.0:8642")
            .await
            .context("failed to bind TCP listener")?;

        println!("Listening on {}", listener.local_addr()?);
        axum::serve(listener, app)
            .await
            .context("error serving axum server")?;

        Ok(())
    }
}
