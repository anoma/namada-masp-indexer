use diesel::{Insertable, Queryable, Selectable};
use serde::Serialize;

use crate::schema::tx;

#[derive(Serialize, Queryable, Selectable, Clone)]
#[diesel(table_name = tx)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TxDb {
    pub id: i32,
    pub block_index: i32,
    pub tx_bytes: Vec<u8>,
    pub block_height: i32,
    pub masp_tx_index: i32,
    pub is_masp_fee_payment: bool,
}

#[derive(Serialize, Insertable, Clone)]
#[diesel(table_name = tx)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TxInsertDb {
    pub block_index: i32,
    pub tx_bytes: Vec<u8>,
    pub block_height: i32,
    pub masp_tx_index: i32,
    pub is_masp_fee_payment: bool,
}
