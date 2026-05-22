use crate::api::routes::create_router;
use crate::domain::traits::{MediaProcessorRepository, SessionRepository, UserRepository};
use crate::infrastructure::database::Database;
use crate::infrastructure::media_processor::MediaProcessorRepositoryImpl;
use crate::infrastructure::packager::service::PackagerService;
use crate::infrastructure::session::SessionRepositoryImpl;
use crate::infrastructure::user::UserRepositoryImpl;
use anyhow::Context;
use axum::Router;
use log::info;
use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::fmt::format;
use std::sync::Arc;
use tower_http::services::ServeDir;

pub struct ServerState {
    pub user_repository: Arc<dyn UserRepository>,
    pub session_repository: Arc<dyn SessionRepository>,
    pub media_processor_repo: Arc<dyn MediaProcessorRepository>,
    pub client_id: String,
    pub client_secret: String,
    pub packager_service: Arc<PackagerService>,
}

pub struct Server {
    state: Arc<ServerState>,
    mdns_daemon: Arc<ServiceDaemon>,
}

impl Server {
    pub fn new(
        db: Arc<Database>,
        packager_service: Arc<PackagerService>,
        client_id: &str,
        client_secret: &str,
    ) -> Self {
        let mdns = ServiceDaemon::new().expect("Failed to create daemon");

        Self {
            state: Arc::new(ServerState {
                session_repository: Arc::new(SessionRepositoryImpl::new(db.clone())),
                user_repository: Arc::new(UserRepositoryImpl::new(db.clone())),
                media_processor_repo: Arc::new(MediaProcessorRepositoryImpl::new(db.clone())),
                client_id: client_id.to_owned(),
                client_secret: client_secret.to_owned(),
                packager_service,
            }),
            mdns_daemon: Arc::new(mdns),
        }
    }

    pub async fn serve(&self) -> anyhow::Result<()> {
        let static_files_service = ServeDir::new("dist")
            .not_found_service(tower_http::services::ServeFile::new("dist/index.html"));

        let app = Router::new()
            .nest("/api", create_router(self.state.clone()))
            .fallback_service(static_files_service);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:8642")
            .await
            .context("failed to bind TCP listener")?;

        let mdns = self.mdns_daemon.clone();
        tokio::spawn(async move {
            let service_type = "_kiroku._tcp.local.";
            let instance_name = "kiroku";
            let ip = format!(
                "{},{}",
                local_ip_address::local_ipv6().unwrap(),
                local_ip_address::local_ip().unwrap()
            );
            let host_name = "kiroku.local.";
            let port = 8642;

            let service =
                ServiceInfo::new(service_type, instance_name, host_name, ip, port, None).unwrap();

            mdns.register(service).unwrap();
            info!("Registering service");
        });

        println!("Listening on {}", listener.local_addr()?);
        axum::serve(listener, app)
            .await
            .context("error serving axum server")?;

        Ok(())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.mdns_daemon.shutdown().ok();
    }
}
