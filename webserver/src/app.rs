use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use axum::error_handling::HandleErrorLayer;
use axum::http::{HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{BoxError, Json, Router};
use axum_trace_id::SetTraceIdLayer;
use lazy_static::lazy_static;
use serde_json::json;
use tower::buffer::BufferLayer;
use tower::limit::RateLimitLayer;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::appstate::AppState;
use crate::config::AppConfig;
use crate::handler;
use crate::state::common::CommonState;

lazy_static! {
    static ref HTTP_TIMEOUT: u64 = 60;
    static ref REQ_PER_SEC: u64 = u64::MAX;
}

pub struct ApplicationServer;

impl ApplicationServer {
    pub async fn serve(config: Arc<AppConfig>) -> anyhow::Result<()> {
        let rps = config.rps.unwrap_or_else(|| *REQ_PER_SEC);
        let db_url = config.database_url.clone();

        let app_state = AppState::new(db_url)?;

        let routes = {
            let common_state = CommonState::new(app_state.clone());

            Router::new()
                .route(
                    "/commitment-tree",
                    get(handler::tree::get_commitment_tree),
                )
                .route(
                    "/witness-map",
                    get(handler::witness_map::get_witness_map),
                )
                .route(
                    "/notes-index",
                    get(handler::notes_index::get_notes_index),
                )
                .route("/tx", get(handler::tx::get_tx))
                .route("/height", get(handler::namada_state::get_latest_height))
                .route(
                    "/block-index",
                    get(handler::namada_state::get_block_index),
                )
                .with_state(common_state)
        };

        let cors = CorsLayer::new()
            .allow_origin("*".parse::<HeaderValue>().unwrap())
            .allow_methods(Any)
            .allow_headers(Any);

        let router = Router::new()
            .nest("/api/v1", routes)
            .merge(Router::new().route(
                "/health",
                get(|| async { env!("VERGEN_GIT_SHA").to_string() }),
            ))
            .with_state(app_state)
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(HandleErrorLayer::new(Self::handle_timeout_error))
                    .timeout(Duration::from_secs(*HTTP_TIMEOUT))
                    .layer(cors)
                    .layer(BufferLayer::new(4096))
                    .layer(RateLimitLayer::new(rps, Duration::from_secs(1)))
                    .layer(SetTraceIdLayer::<String>::new()),
            );

        let router = router.fallback(Self::handle_404);

        let port = config.port;
        let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));

        tracing::info!("🚀 Server has launched on https://{addr}");

        axum::Server::bind(&addr)
            .serve(router.into_make_service())
            .with_graceful_shutdown(Self::shutdown_signal())
            .await
            .context("The server shutdown unexpectedly")?;

        Ok(())
    }

    /// Adds a custom handler for tower's `TimeoutLayer`, see https://docs.rs/axum/latest/axum/middleware/index.html#commonly-used-middleware.
    async fn handle_timeout_error(
        err: BoxError,
    ) -> (StatusCode, Json<serde_json::Value>) {
        if err.is::<tower::timeout::error::Elapsed>() {
            (
                StatusCode::REQUEST_TIMEOUT,
                Json(json!({
                    "error":
                        format!(
                            "request took longer than the configured {} second timeout",
                            *HTTP_TIMEOUT
                        )
                })),
            )
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("unhandled internal error: {}", err)
                })),
            )
        }
    }

    /// Tokio signal handler that will wait for a user to press CTRL+C.
    /// We use this in our hyper `Server` method `with_graceful_shutdown`.
    async fn shutdown_signal() {
        tokio::signal::ctrl_c()
            .await
            .expect("Unexpected tokio shutdown signal await error");
        tracing::info!("Interrupt signal received, shutting down server");
    }

    async fn handle_404() -> impl IntoResponse {
        (
            StatusCode::NOT_FOUND,
            axum::response::Json(serde_json::json!(
                {
                    "error": {
                        "message": "The requested resource does not exist on this server!",
                    }
                }
            )),
        )
    }
}
