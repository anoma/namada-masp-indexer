use diesel::{Insertable, Queryable, Selectable};
use serde::Serialize;

use crate::schema::notes_map;

#[derive(Serialize, Queryable, Selectable, Clone)]
#[diesel(table_name = notes_map)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NotesMapDb {
    pub id: i32,
    pub note_index: i32,
    pub is_fee_unshielding: bool,
    pub note_position: i32,
    pub block_height: i32,
}

#[derive(Serialize, Insertable, Clone)]
#[diesel(table_name = notes_map)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NotesMapInsertDb {
    pub note_index: i32,
    pub is_fee_unshielding: bool,
    pub note_position: i32,
    pub block_height: i32,
}
