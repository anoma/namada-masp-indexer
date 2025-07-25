#![allow(async_fn_in_trait)]

pub mod app;
pub mod appstate;
pub mod config;
pub mod dto;
pub mod error;
pub mod handler;
pub mod repository;
pub mod response;
pub mod service;
pub mod state;
pub mod utils;

use std::sync::Arc;

use clap::Parser;

use crate::app::ApplicationServer;
use crate::config::AppConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Arc::new(AppConfig::parse());

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    ApplicationServer::serve(config).await
}
