use diesel::{Insertable, Queryable, Selectable};
use serde::Serialize;

use crate::schema::notes_map;

#[derive(Serialize, Queryable, Selectable, Clone)]
#[diesel(table_name = notes_map)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NotesMapDb {
    pub block_index: i32,
    pub is_fee_unshielding: bool,
    pub note_position: i32,
    pub block_height: i32,
    pub masp_tx_index: i32,
}

#[derive(Serialize, Insertable, Clone)]
#[diesel(table_name = notes_map)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NotesMapInsertDb {
    pub block_index: i32,
    pub is_fee_unshielding: bool,
    pub note_position: i32,
    pub block_height: i32,
    pub masp_tx_index: i32,
}
