CREATE TYPE USER_ROLE AS ENUM ('admin', 'committee', 'general');

CREATE TABLE users (
    id VARCHAR(64) NOT NULL PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL,
    first_name VARCHAR(128) NOT NULL,
    last_name VARCHAR(128) NOT NULL,
    email VARCHAR(254) UNIQUE NOT NULL,
    role USER_ROLE NOT NULL
);
