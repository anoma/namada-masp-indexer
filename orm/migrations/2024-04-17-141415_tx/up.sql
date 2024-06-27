CREATE TABLE tx (
  id SERIAL PRIMARY KEY,
  note_index INT NOT NULL,
  tx_bytes bytea NOT NULL,
  block_height INT NOT NULL,
  masp_tx_index INT NOT NULL
);

CREATE INDEX tx_block_height_asc ON tx (block_height ASC);
CREATE INDEX tx_block_height_desc ON tx (block_height DESC);
