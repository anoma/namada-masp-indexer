use shared::{
    block_results::{BlockResult, Event}, height::BlockHeight, transaction::Transaction, tx_index::TxIndex
};
use tendermint_rpc::HttpClient;

// Retrieves all the indexes and tx events at the specified height which refer
// to a valid masp transaction. If an index is given, it filters only the
// transactions with an index equal or greater to the provided one.
async fn get_indexed_masp_events_at_height(
    block_results: BlockResult,
    first_idx_to_query: TxIndex,
) -> Vec<(TxIndex, Event)> {
    block_results
        .end_events
        .into_iter()
        .filter_map(|event| {
            let tx_index = event
                .attributes
                .is_valid_masp_tx
                .map(|index| TxIndex(index as u32));
            match tx_index {
                Some(idx) => {
                    if idx.0 >= first_idx_to_query.0 {
                        Some((idx, event))
                    } else {
                        None
                    }
                }
                None => None,
            }
        })
        .collect::<Vec<_>>()
}

/// Extract the relevant shield portions of a [`Tx`], if any.
async fn extract_masp_tx<'args, C: Client + Sync>(
    tx: &Transaction,
    tx_event: &Event
) -> Option<(ExtractedMaspTx, Error)> {
    // We use the changed keys instead of the Transfer object
    // because those are what the masp validity predicate works on
    let (wrapper_changed_keys, changed_keys) = match tx_event.attributes.inner_tx {
        Some(tx_result) => (tx_result.wrapper_changed_keys, tx_result.changed_keys),
        None => (Default::default(), Default::default()),
    };

    let maybe_fee_unshield = if let Some(unshield_fee_tx) = tx.fee_unshielding_tx {
        Some((wrapper_changed_keys, unshield_fee_tx))
    } else {
        None
    };

    // Expect transaction
    let tx_data = tx
        .data()
        .ok_or_else(|| Error::Other("Missing data section".to_string()))?;
    let maybe_masp_tx = match Transfer::try_from_slice(&tx_data) {
        Ok(transfer) => Some((changed_keys, transfer)),
        Err(_) => {
            // This should be a MASP over IBC transaction, it
            // could be a ShieldedTransfer or an Envelope
            // message, need to try both
            extract_payload_from_shielded_action::<C>(&tx_data, action_arg)
                .await
                .ok()
        }
    }
    .map(|(changed_keys, transfer)| {
        if let Some(hash) = transfer.shielded {
            let masp_tx = tx
                .get_section(&hash)
                .ok_or_else(|| {
                    Error::Other(
                        "Missing masp section in transaction".to_string(),
                    )
                })?
                .masp_tx()
                .ok_or_else(|| {
                    Error::Other("Missing masp transaction".to_string())
                })?;

            Ok::<_, Error>(Some((changed_keys, masp_tx)))
        } else {
            Ok(None)
        }
    })
    .transpose()?
    .flatten();

    Ok(ExtractedMaspTx {
        fee_unshielding: maybe_fee_unshield,
        inner_tx: maybe_masp_tx,
    })
}
