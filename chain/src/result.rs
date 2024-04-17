use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum MainError {
    #[error("Masp error: {0}")]
    Masp(String),
    #[error("Tokio error")]
    Tokio,
    #[error("RPC error")]
    Rpc,
    #[error("Database error")]
    Database,
}

#[inline(always)]
pub fn ok<T>(x: T) -> Result<T, MainError> {
    Ok(x)
}

pub trait AsTokioError<T> {
    fn into_tokio_error(self) -> Result<T, MainError>;
}

impl<T> AsTokioError<T> for anyhow::Result<T> {
    #[inline]
    fn into_tokio_error(self) -> Result<T, MainError> {
        self.map_err(|reason| {
            tracing::error!(?reason, "Tokio error");
            MainError::Tokio
        })
    }
}

pub trait AsRpcError<T> {
    fn into_rpc_error(self) -> Result<T, MainError>;
}

impl<T> AsRpcError<T> for anyhow::Result<T> {
    #[inline]
    fn into_rpc_error(self) -> Result<T, MainError> {
        self.map_err(|reason| {
            tracing::error!(?reason, "RPC error");
            MainError::Rpc
        })
    }
}

pub trait AsDbError<T> {
    fn into_db_error(self) -> Result<T, MainError>;
}

impl<T> AsDbError<T> for anyhow::Result<T> {
    #[inline]
    fn into_db_error(self) -> Result<T, MainError> {
        self.map_err(|reason| {
            tracing::error!(?reason, "Database error");
            MainError::Database
        })
    }
}

pub trait ContextDbInteractError<T> {
    fn context_db_interact_error(self) -> anyhow::Result<T>;
}

impl<T, E> ContextDbInteractError<T> for Result<T, E> {
    fn context_db_interact_error(self) -> anyhow::Result<T> {
        self.map_err(|_| anyhow::anyhow!("Failed to interact with db"))
    }
}
