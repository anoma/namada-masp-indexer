// @generated automatically by Diesel CLI.

diesel::table! {
    chain_state (block_height) {
        block_height -> Int4,
    }
}

diesel::table! {
    witness (id) {
        id -> Int4,
        witness_bytes -> Bytea,
        block_height -> Int4,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    chain_state,
    witness,
);
