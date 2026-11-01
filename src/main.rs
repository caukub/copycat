use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{HeaderValue, Method},
    routing::{get, post},
};
use copycat::{
    AppState,
    configuration::get_configuration,
    expiration::delete_expired,
    routes::{index_handler, paste_handler, raw_handler, upload_handler},
    tracing::init_tracing,
};
use std::net::{IpAddr, SocketAddr};
use tokio::signal;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing::{debug, error};

#[tokio::main]
async fn main() {
    init_tracing();

    let configuration = get_configuration().unwrap_or_else(|err| {
        error!("Failed to read configuration: {}", err);
        std::process::exit(1);
    });

    let data_config = configuration.storage.clone();

    std::fs::create_dir_all(data_config.directory.join("metadata")).unwrap_or_else(|err| {
        error!("Failed to create data directory: {}", err);
        std::process::exit(1);
    });

    let app_state = AppState {
        configuration: configuration.clone(),
    };

    let origin = configuration
        .cors
        .allow_origin
        .parse::<HeaderValue>()
        .unwrap_or_else(|err| {
            error!(
                "Failed to convert origin '{}' into HeaderValue: {}",
                configuration.cors.allow_origin, err
            );
            std::process::exit(1);
        });

    let cors = CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([Method::GET, Method::POST]);

    let router = Router::new()
        .route("/", get(index_handler))
        .route("/{id}", get(paste_handler))
        .route("/upload", post(upload_handler))
        .route("/raw/{id}", get(raw_handler))
        .nest_service("/assets", ServeDir::new("./frontend/assets"))
        .with_state(app_state)
        .layer(cors)
        .layer(DefaultBodyLimit::max(configuration.application.body_limit));

    let application = Application {
        host: configuration.application.host,
        port: configuration.application.port,
    };

    let listener = tokio::net::TcpListener::bind(application.address())
        .await
        .unwrap_or_else(|err| {
            error!(
                "Failed to bind TcpListener on {}: {}",
                application.address(),
                err
            );
            std::process::exit(1)
        });

    debug!(
        "Listening on {}",
        listener.local_addr().unwrap_or_else(|err| {
            error!(
                "Failed to get local_addr of TcpListener '{:?}': {}",
                listener, err
            );
            std::process::exit(1);
        })
    );

    tokio::spawn(async move {
        if let Err(err) = delete_expired(&data_config).await {
            error!(
                "Failed to run background task for deleting expired files: {}",
                err
            );
            std::process::exit(1);
        }
    });

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap_or_else(|err| {
            error!("Failed to serve the Axum app: {}", err);
            std::process::exit(1);
        })
}

struct Application {
    host: String,
    port: u16,
}

impl Application {
    fn address(&self) -> SocketAddr {
        let host = self.host.parse::<IpAddr>().unwrap_or_else(|err| {
            error!("Failed to parse '{}' into IpAddr: {}", self.host, err);
            std::process::exit(1);
        });

        SocketAddr::from((host, self.port))
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.unwrap_or_else(|err| {
            error!("Failed to install Ctrl+C handler: {}", err);
            std::process::exit(1);
        });
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .unwrap_or_else(|err| {
                error!("Failed to install signal handler: {}", err);
                std::process::exit(1);
            })
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
