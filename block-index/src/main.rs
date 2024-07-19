pub mod appstate;
pub mod config;

use std::future::{self, Future};
use std::sync::atomic::{self, AtomicBool};
use std::sync::{Arc, Mutex};
use std::task::{Poll, Waker};
use std::time::Duration;

use anyhow::{anyhow, Context};
use clap::Parser;
use deadpool_diesel::postgres::Object;
use orm::schema;
use shared::error::{ContextDbInteractError, IntoMainError, MainError};
use tokio::signal;
use tokio::time::sleep;
use xorf::{BinaryFuse16, Filter};

use crate::appstate::AppState;
use crate::config::AppConfig;

const VERSION_STRING: &str = env!("VERGEN_GIT_SHA");

// TODO: add db migrations for block index

#[tokio::main]
async fn main() -> Result<(), MainError> {
    let AppConfig {
        verbosity,
        database_url,
    } = AppConfig::parse();

    config::install_tracing_subscriber(verbosity);

    tracing::info!(version = VERSION_STRING, "Started the block index builder");
    let mut exit_handle = must_exit();

    let app_state = AppState::new(database_url).into_db_error()?;

    run_migrations(&app_state).await?;

    loop {
        const SLEEP_AMOUNT: Duration = Duration::from_secs(30 * 60);

        tracing::debug!(sleep = ?SLEEP_AMOUNT, "Waiting to build new block index...");

        tokio::select! {
            _ = &mut exit_handle => {
                break;
            }
            _ = sleep(SLEEP_AMOUNT) => {
                _ = build_new_block_index(&app_state).await;
            }
        }
    }

    tracing::info!("Exiting...");
    Ok(())
}

fn must_exit() -> impl Future<Output = ()> {
    let fut_flag = Arc::new(AtomicBool::new(false));
    let task_flag = Arc::clone(&fut_flag);

    let fut_waker = Arc::new(Mutex::new(None));
    let task_waker = Arc::clone(&fut_waker);

    tokio::spawn(async move {
        let mut interrupt =
            signal::unix::signal(signal::unix::SignalKind::interrupt())
                .expect("Failed to install INT signal handler");
        let mut term =
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Failed to install TERM signal handler");
        let mut quit = signal::unix::signal(signal::unix::SignalKind::quit())
            .expect("Failed to install QUIT signal handler");

        tokio::select! {
            _ = interrupt.recv() => {
                tracing::info!("INT signal received");
            }
            _ = term.recv() => {
                tracing::info!("TERM signal received");
            }
            _ = quit.recv() => {
                tracing::info!("QUIT signal received");
            }
        }

        task_flag.store(true, atomic::Ordering::Relaxed);

        let waker: Option<Waker> = task_waker.lock().unwrap().take();
        if let Some(waker) = waker {
            waker.wake();
        }
    });

    future::poll_fn(move |cx| {
        if fut_flag.load(atomic::Ordering::Relaxed) {
            Poll::Ready(())
        } else {
            *fut_waker.lock().unwrap() = Some(cx.waker().clone());
            Poll::Pending
        }
    })
}

async fn run_migrations(app_state: &AppState) -> Result<(), MainError> {
    use diesel_migrations::{
        embed_migrations, EmbeddedMigrations, MigrationHarness,
    };

    const MIGRATIONS: EmbeddedMigrations =
        embed_migrations!("../orm/migrations/");

    async fn run_migrations_inner(conn: Object) -> anyhow::Result<()> {
        tracing::debug!("Running db migrations...");

        conn.interact(|transaction_conn| {
            transaction_conn
                .run_pending_migrations(MIGRATIONS)
                .map_err(|_| anyhow!("Failed to run db migrations"))?;
            anyhow::Ok(())
        })
        .await
        .context_db_interact_error()??;

        tracing::debug!("Finished running db migrations");

        anyhow::Ok(())
    }

    run_migrations_inner(app_state.get_db_connection().await.into_db_error()?)
        .await
        .into_db_error()
}

async fn build_new_block_index(app_state: &AppState) -> Result<(), MainError> {
    use diesel::connection::DefaultLoadingMode as DbDefaultLoadingMode;
    use diesel::prelude::*;
    use schema::tx::dsl::*;

    tracing::info!("Building new block index");

    tracing::debug!("Reading all block heights with masp transactions from db");

    let block_heights = app_state
        .get_db_connection()
        .await
        .into_db_error()?
        .interact(|conn| {
            tx.select(block_height)
                .distinct()
                .load_iter::<_, DbDefaultLoadingMode>(conn)
                .context("Failed to query block heights with masp txs")?
                .try_fold(Vec::new(), |mut accum, maybe_block_height| {
                    tracing::debug!("Reading block height entry from db");
                    let height: i32 = maybe_block_height.context(
                        "Failed to get tx block height row data from db",
                    )?;
                    tracing::debug!("Read block height entry from db");
                    accum.push(height as u64);
                    anyhow::Ok(accum)
                })
        })
        .await
        .context_db_interact_error()
        .into_db_error()?
        .into_db_error()?;

    let block_heights_len = block_heights.len();
    tracing::debug!(
        num_blocks_with_masp_txs = block_heights_len,
        "Read all block heights with masp transactions from db"
    );

    let _serialized_filter = {
        tracing::debug!(
            "Building binary fuse xor filter of all heights with masp \
             transactions"
        );

        let filter: BinaryFuse16 = block_heights
            .try_into()
            .map_err(|err| {
                anyhow!(
                    "Failed to convert queried block heights into binary fuse \
                     xor filter: {err}",
                )
            })
            .into_conversion_error()?;

        let serialized = bincode::serialize(&filter)
            .context(
                "Failed to serialze binary fuse xor filter of block heights",
            )
            .into_serialization_error()?;

        tracing::debug!(
            index_len = filter.len(),
            "Binary fuse xor filter built"
        );

        serialized
    };

    tracing::debug!("Storing binary fuse xor filter in db");
    // TODO: store filter in db
    tracing::debug!("Stored binary fuse xor filter in db");

    tracing::info!(
        num_blocks_with_masp_txs = block_heights_len,
        "Built and stored new block index"
    );

    Ok(())
}
