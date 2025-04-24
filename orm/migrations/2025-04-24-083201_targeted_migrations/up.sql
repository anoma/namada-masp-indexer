-- Your SQL goes here

CREATE TYPE NETWORK_MONIKER AS ENUM (
    'mainnet',
    'housefire',
    'campfire',
    'other'
);

CREATE TABLE network_metadata (
    id SERIAL PRIMARY KEY,
    moniker NETWORK_MONIKER NOT NULL
);
