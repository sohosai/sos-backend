CREATE TABLE files (
    id uuid PRIMARY KEY,
    created_at timestamptz NOT NULL,
    author_id varchar(64) NOT NULL REFERENCES users ON DELETE RESTRICT,
    object_id uuid UNIQUE NOT NULL,
    name varchar(255),
    type_ varchar(255) NOT NULL,
    size bigint NOT NULL
);

CREATE INDEX files_author_id_idx ON files ( author_id );
