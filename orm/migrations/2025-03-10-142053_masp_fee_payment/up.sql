-- Your SQL goes here

ALTER TABLE tx
ADD COLUMN is_masp_fee_payment BOOLEAN NOT NULL DEFAULT false;

ALTER TABLE notes_index
ADD COLUMN is_masp_fee_payment BOOLEAN NOT NULL DEFAULT false;
