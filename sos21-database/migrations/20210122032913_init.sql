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
    display_id varchar(64) UNIQUE NOT NULL,
    owner_id varchar(64) NOT NULL REFERENCES users ON DELETE RESTRICT,
    name varchar(128) NOT NULL,
    kana_name varchar(512) NOT NULL,
    group_name varchar(128) NOT NULL,
    kana_group_name varchar(512) NOT NULL,
    description varchar(4096) NOT NULL,
    category project_category NOT NULL,
    attributes integer NOT NULL
);

CREATE INDEX ON projects ( owner_id );
CREATE INDEX ON projects ( display_id );

CREATE TABLE forms (
    id uuid PRIMARY KEY,
    created_at timestamptz NOT NULL,
    author_id varchar(64) NOT NULL REFERENCES users ON DELETE RESTRICT,
    name varchar(64) NOT NULL,
    description varchar(1024) NOT NULL,
    starts_at timestamptz NOT NULL,
    ends_at timestamptz NOT NULL,
    items bytea NOT NULL,
    condition bytea NOT NULL,
    -- Ac = Academic
    -- Ar = Artistic
    -- Co = Committee
    -- |-----*_query----|
    --  0 0 0 0 0 0 0 0
    --  ∅ AcArAcCoCoCoCo
    --        Ar  AcArAc
    --                Ar
    -- LSB <--------> MSB
    --
    -- The condition is calculated as defined in 'form_query_match' function below.
    unspecified_query bit(8) NOT NULL,
    general_query bit(8) NOT NULL,
    stage_query bit(8) NOT NULL
);

CREATE FUNCTION form_query_match(bit(8), integer)
    RETURNS boolean
    IMMUTABLE LEAKPROOF
    LANGUAGE SQL AS $$
    SELECT $1 & (B'10000000' >> $2::integer) <> B'00000000';
$$;

CREATE TABLE form_condition_includes (
    project_id uuid NOT NULL REFERENCES projects ON DELETE RESTRICT,
    form_id uuid NOT NULL REFERENCES forms ON DELETE RESTRICT,
    UNIQUE (project_id, form_id)
);

CREATE TABLE form_condition_excludes (
    project_id uuid NOT NULL REFERENCES projects ON DELETE RESTRICT,
    form_id uuid NOT NULL REFERENCES forms ON DELETE RESTRICT,
    UNIQUE (project_id, form_id)
);

CREATE TABLE form_answers (
    id uuid PRIMARY KEY,
    created_at timestamptz NOT NULL,
    author_id varchar(64) NOT NULL REFERENCES users ON DELETE RESTRICT,
    form_id uuid NOT NULL REFERENCES forms ON DELETE RESTRICT,
    project_id uuid NOT NULL REFERENCES projects ON DELETE RESTRICT,
    items bytea NOT NULL
);
