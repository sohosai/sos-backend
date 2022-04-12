-- Add migration script here
ALTER TABLE users
DROP CONSTRAINT users_affiliation_category,
DROP COLUMN "affiliation";
