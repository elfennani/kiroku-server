use crate::api::routes::create_router;
use crate::infrastructure::database::connection::Database;
use crate::infrastructure::episode_repo::EpisodeRepository;
use crate::infrastructure::media_repo::MediaRepository;
use crate::infrastructure::packager::service::PackagerService;
use crate::infrastructure::session::SessionRepository;
use anyhow::Context;
use axum::Router;
use log::{debug, info};
use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::services::ServeDir;

pub struct ServerState {
    pub session_repository: Arc<SessionRepository>,
    pub media_repository: Arc<MediaRepository>,
    pub episode_repository: Arc<EpisodeRepository>,
    pub client_id: i32,
    pub client_secret: String,
    pub packager_service: Arc<PackagerService>,
}

pub struct Server {
    state: Arc<ServerState>,
    mdns_daemon: Arc<ServiceDaemon>,
}

pub type RouterState = Arc<ServerState>;
pub type AppRouter = Router<RouterState>;

impl Server {
    pub fn new(
        db: Arc<Database>,
        packager_service: Arc<PackagerService>,
        client_id: i32,
        client_secret: &str,
    ) -> Self {
        let mdns = ServiceDaemon::new().expect("Failed to create daemon");

        Self {
            state: Arc::new(ServerState {
                session_repository: Arc::new(SessionRepository::new(db.clone())),
                media_repository: Arc::new(MediaRepository::new(db.clone())),
                client_id: client_id.to_owned(),
                client_secret: client_secret.to_owned(),
                episode_repository: Arc::new(EpisodeRepository::new(
                    db.clone(),
                    packager_service.app_data_dir(),
                )),
                packager_service,
            }),
            mdns_daemon: Arc::new(mdns),
        }
    }

    pub async fn serve(&self) -> anyhow::Result<()> {
        let static_files_service = ServeDir::new("dist")
            .not_found_service(tower_http::services::ServeFile::new("dist/index.html"));

        debug!(
            "serving assests from {}",
            self.state.packager_service.app_data_dir().to_str().unwrap()
        );
        let app = Router::new()
            .nest("/api", create_router(self.state.clone()))
            .nest_service(
                "/files",
                ServeDir::new(self.state.packager_service.app_data_dir()),
            )
            .fallback_service(static_files_service)
            .into_make_service_with_connect_info::<SocketAddr>();

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
