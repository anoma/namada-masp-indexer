CREATE TABLE commitment_tree (
  id SERIAL PRIMARY KEY,
  tree bytea NOT NULL,
  block_height INT NOT NULL
);

CREATE INDEX commitment_tree_block_height_asc ON commitment_tree (block_height ASC);
CREATE INDEX commitment_tree_block_height_desc ON commitment_tree (block_height DESC);

CREATE INDEX commitment_tree_block_height ON commitment_tree USING HASH (block_height);