CREATE TABLE chain_state (
  block_height SERIAL PRIMARY KEY
);

CREATE INDEX chain_state_block_height_asc ON chain_state (block_height ASC);
CREATE INDEX chain_state_block_height_desc ON chain_state (block_height DESC);