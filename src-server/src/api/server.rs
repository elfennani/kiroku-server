use crate::api::routes::create_router;
use crate::domain::traits::{SessionRepository, UserRepository};
use crate::infrastructure::database::Database;
use crate::infrastructure::packager::service::PackagerService;
use crate::infrastructure::session::SessionRepositoryImpl;
use crate::infrastructure::user::UserRepositoryImpl;
use anyhow::Context;
use axum::Router;
use log::info;
use mdns_sd::{ServiceDaemon, ServiceInfo};
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
                client_id: client_id.to_owned(),
                client_secret: client_secret.to_owned(),
                packager_service,
            }),
            mdns_daemon: Arc::new(mdns),
        }
    }

    pub async fn serve(&self) -> anyhow::Result<()> {
        let app = Router::new().nest("/api", create_router(self.state.clone()));

        let listener = tokio::net::TcpListener::bind("0.0.0.0:8642")
            .await
            .context("failed to bind TCP listener")?;

        let mdns = self.mdns_daemon.clone();
        tokio::spawn(async move {
            let service_type = "_kiroku._udp.local.";
            let instance_name = "kiroku";
            let ip = local_ip_address::local_ip().unwrap().to_string();
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
