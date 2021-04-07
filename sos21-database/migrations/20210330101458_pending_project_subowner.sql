CREATE TABLE pending_projects (
    id uuid PRIMARY KEY,
    created_at timestamptz NOT NULL,
    author_id varchar(64) NOT NULL REFERENCES users ON DELETE RESTRICT,
    name varchar(128) NOT NULL,
    kana_name varchar(512) NOT NULL,
    group_name varchar(128) NOT NULL,
    kana_group_name varchar(512) NOT NULL,
    description varchar(4096) NOT NULL,
    category project_category NOT NULL,
    attributes integer NOT NULL
);

INSERT INTO pending_projects
SELECT
    id,
    now() AS created_at,
    owner_id AS author_id,
    name,
    kana_name,
    group_name,
    kana_group_name,
    description,
    category,
    attributes
FROM projects;

DELETE FROM projects;

ALTER TABLE projects ADD COLUMN subowner_id varchar(64) NOT NULL REFERENCES users ON DELETE RESTRICT;
