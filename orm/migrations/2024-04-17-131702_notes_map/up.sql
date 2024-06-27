CREATE TABLE notes_map (
  note_position INT PRIMARY KEY,
  note_index INT NOT NULL,
  is_fee_unshielding BOOLEAN NOT NULL,
  block_height INT NOT NULL,
  masp_tx_index INT NOT NULL
);

CREATE INDEX notes_map_block_height_asc ON notes_map (block_height ASC);
CREATE INDEX notes_map_block_height_desc ON notes_map (block_height DESC);

CREATE INDEX notes_map_block_height ON notes_map USING HASH (block_height);
