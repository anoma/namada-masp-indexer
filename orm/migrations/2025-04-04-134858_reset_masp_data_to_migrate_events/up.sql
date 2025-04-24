-- Your SQL goes here

DELETE FROM block_index;
UPDATE chain_state SET block_height = 675070;
DELETE FROM commitment_tree WHERE block_height > 675070;
DELETE FROM notes_index WHERE block_height > 675070;
DELETE FROM tx WHERE block_height > 675070;
DELETE FROM witness WHERE block_height > 675070;
