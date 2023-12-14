extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate redis;

use axum::routing::{get, post, delete, put};
use axum::extract::Extension;
use axum::middleware;
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::request_id::MakeRequestUuid;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tower_http::ServiceBuilderExt;
use mongodb::{bson::doc, options::ClientOptions, Client};

mod api;

#[tokio::main]
async fn main() {
    pretty_env_logger::init_timed();

    info!("Staring connect to mongodb.");
    // Configuration mongodb
    let client_options =
        ClientOptions::parse("mongodb://admin:admin@localhost/")
            .await
            .expect("Can't connect to mongodb.");

    let client = Client::with_options(client_options).unwrap();
    let db = client.database("toxin");

    db.run_command(doc! {"ping": 1}, None)
        .await
        .expect("Can't ping to mongodb.");

    info!("Connected to mongodb.");

    info!("Starting connect to redis.");
    let redis_client = redis::Client::open("redis://127.0.0.1/").unwrap();
    info!("Connected to redis.");

    // Shared state
    let shared_state = Arc::new(api::state::State {
        db,
        redis_client
    });

    let private_route = Router::new()
        .route("/api/v1/sign-out", post(api::auth::sign_in))
        .route("/api/v1/accounts", get(api::account::account_list))
        .route("/api/v1/accounts", post(api::account::account_create))
        .route("/api/v1/accounts/:id", get(api::account::account_detail))        
        .route("/api/v1/accounts/:id", delete(api::account::account_remove))
        .route("/api/v1/accounts/:id", put(api::account::account_update))
        .route_layer(middleware::from_fn(api::middleware::auth));

    let public_route: Router = Router::new()
        .route("/", get(root))
        .route("/ping", get(api::ping::ping))
        .route("/api/v1/sign-in", post(api::auth::sign_in));

    let app = Router::new()
        .merge(private_route)
        .merge(public_route)
        .layer(
            ServiceBuilder::new()
                .set_x_request_id(MakeRequestUuid)
                .propagate_x_request_id()
                .layer(TraceLayer::new_for_http())
        )
        .layer(Extension(shared_state));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    info!("Server run 127.0.0.1:3000");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}


async fn root() -> String {
    info!("Root entrypoint");
    String::from("Welcome to Toxin!")
}
