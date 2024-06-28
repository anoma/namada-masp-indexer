use diesel::{Insertable, Queryable, Selectable};
use serde::Serialize;

use crate::schema::chain_state;

#[derive(Serialize, Queryable, Selectable, Clone)]
#[diesel(table_name = chain_state)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChainStateDb {
    pub block_height: i32,
}

#[derive(Serialize, Insertable, Clone)]
#[diesel(table_name = chain_state)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChainStateteInsertDb {
    pub id: i32,
    pub block_height: i32,
}
