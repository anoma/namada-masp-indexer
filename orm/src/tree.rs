use diesel::{Insertable, Queryable, Selectable};
use serde::Serialize;

use crate::schema::commitment_tree;

#[derive(Serialize, Queryable, Selectable, Clone)]
#[diesel(table_name = commitment_tree)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TreeDb {
    pub id: i32,
    pub tree: Vec<u8>,
    pub block_height: i32,
}

#[derive(Serialize, Insertable, Clone)]
#[diesel(table_name = commitment_tree)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TreeInsertDb {
    pub tree: Vec<u8>,
    pub block_height: i32,
}
