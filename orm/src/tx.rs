use diesel::{Insertable, Queryable, Selectable};
use serde::Serialize;

use crate::schema::tx;

#[derive(Serialize, Queryable, Selectable, Clone)]
#[diesel(table_name = tx)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TxDb {
    pub id: i32,
    pub note_index: i32,
    pub tx_bytes: Vec<u8>,
    pub block_height: i32,
}

#[derive(Serialize, Insertable, Clone)]
#[diesel(table_name = tx)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TxInsertDb {
    pub note_index: i32,
    pub tx_bytes: Vec<u8>,
    pub block_height: i32,
}
