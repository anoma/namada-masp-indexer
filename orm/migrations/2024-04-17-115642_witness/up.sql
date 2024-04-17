CREATE TABLE witness (
  id SERIAL PRIMARY KEY,
  witness_bytes bytea NOT NULL,
  witness_idx INT NOT NULL,
  block_height INT NOT NULL
);

CREATE INDEX witness_block_height_asc ON witness (block_height ASC);
CREATE INDEX witness_block_height_desc ON witness (block_height DESC);

CREATE INDEX witness_block_height ON witness USING HASH (block_height);