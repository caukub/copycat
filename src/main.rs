use axum::extract::DefaultBodyLimit;
use axum::http::{HeaderValue, Method};
use axum::routing::{get, post};
use axum::{middleware, Router};
use copycat::configuration::{get_configuration, Settings};
use copycat::middleware::{api_admin_middleware, api_middleware};
use copycat::paste::analyzer::PasteAnalyzer;
use copycat::redis::get_redis_connection;
use copycat::routes::api::{
    all::get_api_all, frontend::detail::get_frontend_api_detail, frontend::paste::get_paste,
    leaks::get_api_leaks, plugins::get_api_plugins, ports::get_api_ports,
};
use copycat::routes::{
    get::raw::get_raw,
    post::{get_expiration, upload::post_upload},
};
use copycat::{AppState, RedisState};
use fred::{clients::RedisPool, interfaces::KeysInterface};
use mclog::analyzer::dynamic::{ScriptPlatform, SCRIPTS_DIRECTORY};
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer};
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    let configuration = get_configuration().expect("Failed to read configuration");
    let redis_pool = get_redis_connection(&configuration)
        .await
        .expect("Failed to connect to Redis server");

    Initializer::new(redis_pool.clone(), configuration.clone())
        .init()
        .await;

    let app = Application::new(&configuration);

    let cors_layer = CorsLayer::new()
        .allow_origin(
            configuration
                .cors
                .allow_origin
                .parse::<HeaderValue>()
                .expect("Couldn't parse allow_origin from config into HeaderValue"),
        )
        .allow_methods([Method::GET, Method::POST]);

    let redis_state = RedisState { pool: redis_pool };

    let app_state = AppState {
        configuration: configuration.clone(),
        redis_state,
    };

    let frontend_api_router = Router::new()
        .route("/paste/:id", get(get_paste))
        .route("/detail/:detail_id", get(get_frontend_api_detail))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            api_admin_middleware,
        ));

    let api_router = Router::new()
        .route("/all/:id", get(get_api_all))
        .route("/plugins/:id", get(get_api_plugins))
        .route("/ports/:id", get(get_api_ports))
        .route("/leaks/:id", get(get_api_leaks))
        //.nest("/admin", admin_api_router)
        .nest("/frontend", frontend_api_router)
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            api_middleware,
        ));

    let router = Router::new()
        .route("/raw/:id", get(get_raw))
        .route("/upload", post(post_upload))
        .nest("/api", api_router);

    let router = router
        .layer(cors_layer)
        .layer(DefaultBodyLimit::max(configuration.application.body_limit))
        .layer(RequestBodyLimitLayer::new(
            configuration.application.body_limit,
        ))
        .with_state(app_state);

    let address = app.address();

    debug!("listening on {}", address);

    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();

    axum::serve(listener, router).await.unwrap();
}

struct Application {
    host: String,
    port: u16,
}

impl Application {
    fn new(configuration: &Settings) -> Self {
        Self {
            host: configuration.application.host.clone(),
            port: configuration.application.port,
        }
    }

    fn address(&self) -> SocketAddr {
        let addr = format!("{}:{}", self.host, self.port);

        let addr: SocketAddr = addr
            .parse()
            .unwrap_or_else(|_| panic!("Couldn't convert '{}' to SocketAddr", addr));

        addr
    }
}

struct Initializer {
    pool: RedisPool,
    configuration: Settings,
}

impl Initializer {
    fn new(redis_pool: RedisPool, configuration: Settings) -> Self {
        Self {
            pool: redis_pool,
            configuration,
        }
    }

    async fn init(&mut self) {
        self.init_tracing();
        self.init_directories();
        self.generate_content_types().await;
    }

    fn init_tracing(&self) {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                    "copycat=debug,tower_http=debug,axum::rejection=trace".into()
                }),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    fn init_directories(&self) {
        self.init_data_dir();
        self.init_scripts_dir();
    }

    fn init_data_dir(&self) {
        let data_directory = &self.configuration.storage.directory;
        if !data_directory.exists() {
            std::fs::create_dir_all(data_directory).expect("Data directory couldn't be created");
        }
    }

    fn init_scripts_dir(&self) {
        if !SCRIPTS_DIRECTORY.exists() {
            std::fs::create_dir_all(SCRIPTS_DIRECTORY.as_path())
                .expect("Scripts directory couldn't be created")
        }

        for platform in ScriptPlatform::iter() {
            let dir = SCRIPTS_DIRECTORY.join(platform.directory());

            if !dir.exists() {
                std::fs::create_dir_all(dir)
                    .expect("Script platform directory couldn't be created");
            }
        }
    }

    async fn generate_content_types(&mut self) {
        for file in self
            .configuration
            .storage
            .directory
            .read_dir()
            .expect("Couldn't read storage directory")
        {
            let file = file.unwrap();
            let file_name = file
                .file_name()
                .to_str()
                .expect("Failed reading file name to str")
                .to_string();

            let paste_type: Option<String> = self.pool.get(file_name.clone()).await.unwrap_or(None);

            if paste_type.is_none() {
                let content = tokio::fs::read_to_string(file.path()).await.unwrap();
                let content_as_bytes = content.as_bytes();

                let paste_analyzer = PasteAnalyzer::new();
                let paste_type = paste_analyzer.paste_type(content_as_bytes);

                let _: () = self
                    .pool
                    .set(
                        file_name.clone(),
                        paste_type,
                        get_expiration(&self.configuration),
                        None,
                        false,
                    )
                    .await
                    .unwrap();
            }
        }
    }
}
