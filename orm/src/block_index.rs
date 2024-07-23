use diesel::{Insertable, Queryable, Selectable};

use crate::schema::block_index;

#[derive(Insertable, Queryable, Selectable, Clone)]
#[diesel(table_name = block_index)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BlockIndex {
    pub id: i32,
    pub serialized_data: Vec<u8>,
}
