// @generated automatically by Diesel CLI.

diesel::table! {
    block_index (id) {
        id -> Int4,
        serialized_data -> Bytea,
        block_height -> Int4,
    }
}

diesel::table! {
    chain_state (id) {
        id -> Int4,
        block_height -> Int4,
    }
}

diesel::table! {
    commitment_tree (id) {
        id -> Int4,
        tree -> Bytea,
        block_height -> Int4,
    }
}

diesel::table! {
    notes_index (note_position) {
        note_position -> Int4,
        block_index -> Int4,
        is_fee_unshielding -> Bool,
        block_height -> Int4,
        masp_tx_index -> Int4,
    }
}

diesel::table! {
    tx (id) {
        id -> Int4,
        block_index -> Int4,
        tx_bytes -> Bytea,
        block_height -> Int4,
        masp_tx_index -> Int4,
    }
}

diesel::table! {
    witness (id) {
        id -> Int4,
        witness_bytes -> Bytea,
        witness_idx -> Int4,
        block_height -> Int4,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    block_index,
    chain_state,
    commitment_tree,
    notes_index,
    tx,
    witness,
);
