pub mod appstate;
pub mod config;

use std::future::{self, Future};
use std::num::NonZeroU64;
use std::ops::ControlFlow;
use std::sync::atomic::{self, AtomicBool};
use std::sync::{Arc, Mutex};
use std::task::{Poll, Waker};
use std::time::Duration;

use anyhow::{Context, anyhow};
use clap::Parser;
use deadpool_diesel::postgres::Object;
use orm::block_index::BlockIndex;
use orm::schema;
use shared::error::{ContextDbInteractError, IntoMainError, MainError};
use tokio::signal;
use tokio::time::sleep;
use xorf::{BinaryFuse16, Filter};

use crate::appstate::AppState;
use crate::config::AppConfig;

const VERSION_STRING: &str = env!("VERGEN_GIT_SHA");

macro_rules! exit {
    () => {{
        tracing::info!("Exiting...");
        return Ok(());
    }};
}

#[tokio::main(worker_threads = 2)]
async fn main() -> Result<(), MainError> {
    let AppConfig {
        verbosity,
        database_url,
        interval,
    } = AppConfig::parse();

    let (non_blocking_logger, _worker) =
        tracing_appender::non_blocking(std::io::stdout());
    config::install_tracing_subscriber(verbosity, non_blocking_logger);

    tracing::info!(version = VERSION_STRING, "Started the block index builder");
    let mut exit_handle = must_exit();

    let app_state = AppState::new(database_url).await.into_db_error()?;

    if wait_for_migrations(&mut exit_handle, &app_state)
        .await
        .is_break()
    {
        exit!();
    }
    build_block_indexes(&mut exit_handle, interval, &app_state).await;

    exit!();
}

async fn wait_for_migrations<F>(
    mut exit_handle: F,
    app_state: &AppState,
) -> ControlFlow<()>
where
    F: Future<Output = ()> + Unpin,
{
    while run_migrations(app_state).await.is_err() {
        const SLEEP_AMOUNT: Duration = Duration::from_secs(5);

        tracing::info!(after = ?SLEEP_AMOUNT, "Retrying migrations");

        tokio::select! {
            _ = &mut exit_handle => {
                return ControlFlow::Break(());
            }
            _ = sleep(SLEEP_AMOUNT) => {}
        }
    }

    ControlFlow::Continue(())
}

async fn build_block_indexes<F>(
    mut exit_handle: F,
    interval: Option<NonZeroU64>,
    app_state: &AppState,
) where
    F: Future<Output = ()> + Unpin,
{
    const DEFAULT_SLEEP_AMOUNT: Duration = Duration::from_secs(30 * 60);
    let sleep_amount = interval
        .map(|interval| Duration::from_secs(interval.get()))
        .unwrap_or(DEFAULT_SLEEP_AMOUNT);

    loop {
        tracing::debug!(after = ?sleep_amount, "Building new block index");

        tokio::select! {
            _ = &mut exit_handle => {
                return;
            }
            _ = sleep(sleep_amount) => {
                _ = build_new_block_index(app_state).await;
            }
        }
    }
}

fn must_exit() -> impl Future<Output = ()> {
    struct ExitHandle {
        flag: AtomicBool,
        waker: Mutex<Option<Waker>>,
    }

    let fut_handle = Arc::new(ExitHandle {
        flag: AtomicBool::new(false),
        waker: Mutex::new(None),
    });
    let task_handle = Arc::clone(&fut_handle);

    tokio::spawn(async move {
        let mut interrupt =
            signal::unix::signal(signal::unix::SignalKind::interrupt())
                .expect("Failed to install INT signal handler");
        let mut term =
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Failed to install TERM signal handler");
        let mut quit = signal::unix::signal(signal::unix::SignalKind::quit())
            .expect("Failed to install QUIT signal handler");

        let signal_descriptor = tokio::select! {
            _ = interrupt.recv() => "INT",
            _ = term.recv() => "TERM",
            _ = quit.recv() => "QUIT",
        };
        tracing::info!(which = signal_descriptor, "Signal received");

        atomic::fence(atomic::Ordering::Release);
        task_handle.flag.store(true, atomic::Ordering::Relaxed);

        let waker = task_handle.waker.lock().unwrap().take();
        if let Some(waker) = waker {
            waker.wake();
        }
    });

    future::poll_fn(move |cx| {
        if fut_handle.flag.load(atomic::Ordering::Relaxed) {
            atomic::fence(atomic::Ordering::Acquire);
            Poll::Ready(())
        } else {
            *fut_handle.waker.lock().unwrap() = Some(cx.waker().clone());
            Poll::Pending
        }
    })
}

async fn run_migrations(app_state: &AppState) -> Result<(), MainError> {
    use diesel_migrations::{
        EmbeddedMigrations, MigrationHarness, embed_migrations,
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

    tracing::info!("Starting new masp txs block index");

    tracing::debug!("Reading all block heights with masp transactions from db");

    let conn = app_state.get_db_connection().await.into_db_error()?;

    let (last_height, block_heights_with_txs) = conn
        .interact(|conn| {
            conn.build_transaction().read_only().run(|conn| {
                let last_height = {
                    use diesel::prelude::OptionalExtension;
                    use schema::chain_state::dsl::*;

                    chain_state
                        .select(block_height)
                        .first(conn)
                        .optional()
                        .context("Failed to query last block height")
                }?;

                let block_heights_with_txs = {
                    use schema::tx::dsl::*;

                    tx.select(block_height)
                        .distinct()
                        .load_iter::<_, DbDefaultLoadingMode>(conn)
                        .context("Failed to query block heights with masp txs")?
                        .try_fold(
                            Vec::new(),
                            |mut accum, maybe_block_height| {
                                tracing::debug!(
                                    "Reading block height entry from db"
                                );
                                let height: i32 = maybe_block_height.context(
                                    "Failed to get tx block height row data \
                                     from db",
                                )?;
                                tracing::debug!(
                                    "Read block height entry from db"
                                );
                                accum.push(u64::try_from(height).context(
                                    "Failed to convert block height from i32 \
                                     to u64",
                                )?);
                                anyhow::Ok(accum)
                            },
                        )
                }?;

                anyhow::Ok((last_height, block_heights_with_txs))
            })
        })
        .await
        .context_db_interact_error()
        .into_db_error()?
        .into_db_error()?;

    let block_heights_len = block_heights_with_txs.len();
    tracing::debug!(
        num_blocks_with_masp_txs = block_heights_len,
        "Read all block heights with masp transactions from db"
    );

    let serialized_filter = tokio::task::block_in_place(|| {
        tracing::debug!(
            "Building binary fuse xor filter of all heights with masp \
             transactions"
        );

        let filter: BinaryFuse16 = block_heights_with_txs
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

        Ok(serialized)
    })?;

    tracing::debug!("Storing binary fuse xor filter in db");

    conn.interact(move |conn| {
        use schema::block_index::dsl::*;

        let db_filter = BlockIndex {
            id: 0,
            serialized_data: serialized_filter,
            block_height: last_height.unwrap_or_default(),
        };

        diesel::insert_into(block_index)
            .values(&db_filter)
            .on_conflict(id)
            .do_update()
            .set((
                block_height.eq(&db_filter.block_height),
                serialized_data.eq(&db_filter.serialized_data),
            ))
            .execute(conn)
            .context("Failed to insert masp txs block index into db")?;

        anyhow::Ok(())
    })
    .await
    .context_db_interact_error()
    .into_db_error()?
    .into_db_error()?;

    tracing::debug!("Stored binary fuse xor filter in db");

    tracing::info!(
        num_blocks_with_masp_txs = block_heights_len,
        "Built and stored new masp txs block index"
    );

    Ok(())
}
