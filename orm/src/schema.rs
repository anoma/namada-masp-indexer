// @generated automatically by Diesel CLI.

diesel::table! {
    chain_state (block_height) {
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
    notes_map (id) {
        id -> Int4,
        note_index -> Int4,
        is_fee_unshielding -> Bool,
        note_position -> Int4,
        block_height -> Int4,
    }
}

diesel::table! {
    tx (id) {
        id -> Int4,
        note_index -> Int4,
        tx_bytes -> Bytea,
        block_height -> Int4,
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
    chain_state,
    commitment_tree,
    notes_map,
    tx,
    witness,
);
