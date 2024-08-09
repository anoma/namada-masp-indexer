CREATE TABLE block_index (
  id SERIAL PRIMARY KEY,
  -- NB: serialized with `bincode`
  serialized_data bytea NOT NULL,
  block_height INT NOT NULL
);
