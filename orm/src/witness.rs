use diesel::{Insertable, Queryable, Selectable};
use serde::Serialize;

use crate::schema::witness;

#[derive(Serialize, Queryable, Selectable, Clone)]
#[diesel(table_name = witness)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WitnessDb {
    pub id: i32,
    pub witness_idx: i32,
    pub block_height: i32,
    pub witness_bytes: Vec<u8>,
}

#[derive(Serialize, Insertable, Clone)]
#[diesel(table_name = witness)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WitnessInsertDb {
    pub witness_bytes: Vec<u8>,
    pub witness_idx: i32,
    pub block_height: i32,
}
