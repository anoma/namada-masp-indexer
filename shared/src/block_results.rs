use std::str::FromStr;

use namada_sdk::events::EventType;
use namada_sdk::events::extend::ReadFromEventAttributes;
use namada_tx::IndexedTx;
use namada_tx::event::{MaspEvent, MaspEventKind, MaspTxRef};
use tendermint_rpc::endpoint::block_results;

pub fn locate_masp_txs(
    raw_block_results: &block_results::Response,
) -> Vec<MaspEvent> {
    raw_block_results
        .end_block_events
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|event| {
            // Check if the event is a Masp one
            let Ok(kind) = EventType::from_str(&event.kind) else {
                return None;
            };
            let kind = if kind == namada_tx::event::masp_types::TRANSFER {
                MaspEventKind::Transfer
            } else if kind == namada_tx::event::masp_types::FEE_PAYMENT {
                MaspEventKind::FeePayment
            } else {
                return None;
            };

            // Extract the data from the event's attributes
            let Ok(data) =
                MaspTxRef::read_from_event_attributes(&event.attributes)
            else {
                return None;
            };
            let Ok(tx_index) =
                IndexedTx::read_from_event_attributes(&event.attributes)
            else {
                return None;
            };

            Some(MaspEvent {
                tx_index,
                kind,
                data,
            })
        })
        .collect()
}
