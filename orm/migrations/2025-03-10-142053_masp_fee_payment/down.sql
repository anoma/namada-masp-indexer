-- This file should undo anything in `up.sql`

ALTER TABLE tx DROP COLUMN is_masp_fee_payment;
ALTER TABLE notes_index DROP COLUMN is_masp_fee_payment;
