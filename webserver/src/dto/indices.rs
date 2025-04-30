use std::collections::{HashMap, HashSet};

use axum::extract::RawQuery;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::error::tx::TxError;

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct IndexQueryParams {
    #[validate(length(min = 1))]
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
        let args = ArgParser::parse(&query).map_err(TxError::RawQuery)?;
        let heights = args.0.get("heights").ok_or_else(|| {
            TxError::RawQuery("Expected argument name `heights`".to_string())
        })?;
        let block_indices = args.0.get("block_indices").ok_or_else(|| {
            TxError::RawQuery(
                "Expected argument name `block_indices`".to_string(),
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
            let mut distinct_heights = HashSet::new();
            let parsed = Self {
                indices: heights
                    .into_iter()
                    .zip(block_indices)
                    .map(|(h, ix)| {
                        let ix = Index {
                            height: h,
                            block_index: ix,
                        };
                        distinct_heights.insert(h);
                        ix.validate()
                            .map_err(|e| TxError::Validation(e.to_string()))
                            .map(|_| ix)
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            };
            if distinct_heights.len() > 30 {
                Err(TxError::Validation(
                    "Cannot request more than 30 unique block heights"
                        .to_string(),
                ))
            } else {
                parsed
                    .validate()
                    .map_err(|e| TxError::Validation(e.to_string()))?;
                Ok(parsed)
            }
        }
    }
}

#[derive(Default)]
struct ArgParser<'a>(HashMap<&'a str, &'a str>);

impl<'b> ArgParser<'b> {
    fn parse<'a>(input: &'a str) -> Result<ArgParser<'b>, String>
    where
        'a: 'b,
    {
        let mut args = Self::default();
        for kv_pair in input.split('&') {
            if let Ok([k, v]) =
                <[&str; 2]>::try_from(kv_pair.split('=').collect::<Vec<&str>>())
            {
                args.0.insert(k, v);
            } else {
                return Err("Could not parse one of the arguments with \
                            expected format key=value"
                    .to_string());
            }
        }
        Ok(args)
    }
}
