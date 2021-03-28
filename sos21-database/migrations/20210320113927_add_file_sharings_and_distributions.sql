CREATE TYPE file_sharing_scope AS ENUM ('project', 'form_answer', 'committee', 'committee_operator', 'public');

CREATE TABLE file_sharings (
    id uuid PRIMARY KEY,
    created_at timestamptz NOT NULL,
    file_id uuid NOT NULL REFERENCES files ON DELETE RESTRICT,
    is_revoked boolean NOT NULL,
    expires_at timestamptz,
    scope file_sharing_scope NOT NULL,
    project_id uuid REFERENCES projects ON DELETE RESTRICT,
    form_answer_project_id uuid REFERENCES projects ON DELETE RESTRICT,
    form_answer_form_id uuid REFERENCES forms ON DELETE RESTRICT,
    CONSTRAINT file_sharings_scope_project_id CHECK ((scope = 'project') = (project_id IS NOT NULL)),
    CONSTRAINT file_sharings_scope_form_answer_project_id CHECK ((scope = 'form_answer') = (form_answer_project_id IS NOT NULL)),
    CONSTRAINT file_sharings_scope_form_answer_form_id CHECK ((scope = 'form_answer') = (form_answer_form_id IS NOT NULL))
);

CREATE TABLE file_distributions (
    id uuid PRIMARY KEY,
    created_at timestamptz NOT NULL,
    author_id varchar(64) NOT NULL REFERENCES users ON DELETE RESTRICT,
    name varchar(64) NOT NULL,
    description varchar(1024) NOT NULL
);

CREATE TABLE file_distribution_files (
    distribution_id uuid NOT NULL REFERENCES file_distributions ON DELETE RESTRICT,
    project_id uuid NOT NULL REFERENCES projects ON DELETE RESTRICT,
    sharing_id uuid NOT NULL REFERENCES file_sharings ON DELETE RESTRICT,
    UNIQUE (distribution_id, project_id)
);
