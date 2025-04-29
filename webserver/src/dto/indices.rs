use axum::extract::RawQuery;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::error::tx::TxError;

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct IndexQueryParams {
    #[validate(length(min = 1, max = 30))]
    pub indices: Vec<Index>,
}

#[derive(Copy, Clone, Serialize, Deserialize, Validate)]
pub struct Index {
    #[validate(range(min = 1))]
    pub height: u64,
    #[validate(range(min = 0))]
    pub block_index: u32,
}

impl TryFrom<RawQuery> for IndexQueryParams {
    type Error = TxError;

    fn try_from(raw: RawQuery) -> Result<Self, Self::Error> {
        let Some(query) = raw.0 else {
            return Err(Self::Error::RawQuery(
                "Received empty indices".to_string(),
            ));
        };

        let mut parts = query.split('&');
        let heights = parts
            .next()
            .ok_or_else(|| {
                TxError::RawQuery(
                    "Indices argument requires heights and block_indices \
                     separated by an '&'"
                        .to_string(),
                )
            })?
            .strip_prefix("heights=")
            .ok_or_else(|| {
                TxError::RawQuery(
                    "Expected argument name `heights=`".to_string(),
                )
            })?;
        let block_indices = parts
            .next()
            .ok_or_else(|| {
                TxError::RawQuery(
                    "Indices argument requires heights and block_indices \
                     separated by an '&'"
                        .to_string(),
                )
            })?
            .strip_prefix("block_indices=")
            .ok_or_else(|| {
                TxError::RawQuery(
                    "Expected argument name `block_indices=`".to_string(),
                )
            })?;
        let heights = heights
            .split('.')
            .map(|s| {
                s.parse::<u64>().map_err(|_| {
                    TxError::RawQuery(format!(
                        "Could not parse {s} as block height"
                    ))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let block_indices = block_indices
            .split('.')
            .map(|s| {
                s.parse::<u32>().map_err(|_| {
                    TxError::RawQuery(format!(
                        "Could not parse {s} as block index"
                    ))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if heights.len() != block_indices.len() {
            Err(TxError::RawQuery(
                "Number of block heights and block indices must be equal"
                    .to_string(),
            ))
        } else {
            let parsed = Self {
                indices: heights
                    .into_iter()
                    .zip(block_indices)
                    .map(|(h, ix)| {
                        let ix = Index {
                            height: h,
                            block_index: ix,
                        };
                        ix.validate().map_err(TxError::Validation).map(|_| ix)
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            };
            parsed.validate().map_err(TxError::Validation)?;
            Ok(parsed)
        }
    }
}
