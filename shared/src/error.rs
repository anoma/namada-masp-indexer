use std::fmt;

#[derive(PartialEq, Eq)]
pub struct MainError;

impl fmt::Debug for MainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "namada-masp-indexer shut down unexpectedly")
    }
}

impl fmt::Display for MainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

impl std::error::Error for MainError {}

#[inline(always)]
pub fn ok<T>(x: T) -> Result<T, MainError> {
    Ok(x)
}

pub trait IntoMainError<T>: Sized {
    fn into_main_error(self, description: &str) -> Result<T, MainError>;

    #[inline]
    fn into_rpc_error(self) -> Result<T, MainError> {
        self.into_main_error("RPC error")
    }

    #[inline]
    fn into_db_error(self) -> Result<T, MainError> {
        self.into_main_error("Database error")
    }

    #[inline]
    fn into_masp_error(self) -> Result<T, MainError> {
        self.into_main_error("MASP error")
    }
}

impl<T> IntoMainError<T> for anyhow::Result<T> {
    #[inline]
    fn into_main_error(self, description: &str) -> Result<T, MainError> {
        self.map_err(|reason| {
            tracing::error!(?reason, "{description}");
            MainError
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
