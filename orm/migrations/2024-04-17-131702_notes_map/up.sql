CREATE TABLE notes_index (
  note_position INT PRIMARY KEY,
  block_index INT NOT NULL,
  is_fee_unshielding BOOLEAN NOT NULL,
  block_height INT NOT NULL,
  masp_tx_index INT NOT NULL
);

CREATE INDEX notes_index_block_height_asc ON notes_index (block_height ASC);
CREATE INDEX notes_index_block_height_desc ON notes_index (block_height DESC);

CREATE INDEX notes_index_block_height ON notes_index USING HASH (block_height);
