use diesel::{Insertable, Queryable, Selectable};
use serde::Serialize;

use crate::schema::notes_index;

#[derive(Serialize, Queryable, Selectable, Clone)]
#[diesel(table_name = notes_index)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NotesIndexDb {
    pub block_index: i32,
    pub note_position: i32,
    pub block_height: i32,
    pub masp_tx_index: i32,
}

#[derive(Serialize, Insertable, Clone)]
#[diesel(table_name = notes_index)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NotesIndexInsertDb {
    pub block_index: i32,
    pub note_position: i32,
    pub block_height: i32,
    pub masp_tx_index: i32,
}
