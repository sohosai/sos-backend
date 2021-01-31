CREATE TYPE user_role AS ENUM ('administrator', 'committee_operator', 'committee', 'general');

CREATE TABLE users (
    id varchar(64) PRIMARY KEY,
    created_at timestamptz NOT NULL,
    first_name varchar(64) NOT NULL,
    kana_first_name varchar(256) NOT NULL,
    last_name varchar(64) NOT NULL,
    kana_last_name varchar(256) NOT NULL,
    phone_number varchar(16) NOT NULL,
    affiliation varchar(128) NOT NULL,
    email varchar(128) NOT NULL,
    role user_role NOT NULL
);

CREATE TYPE project_category AS ENUM ('general', 'stage');

CREATE TABLE projects (
    id uuid PRIMARY KEY,
    created_at timestamptz NOT NULL,
    owner_id varchar(64) NOT NULL REFERENCES users ON DELETE RESTRICT,
    name varchar(128) NOT NULL,
    kana_name varchar(512) NOT NULL,
    group_name varchar(128) NOT NULL,
    kana_group_name varchar(512) NOT NULL,
    description varchar(4096) NOT NULL,
    category project_category NOT NULL,
    attributes bigint NOT NULL
);

CREATE INDEX ON projects ( owner_id );
