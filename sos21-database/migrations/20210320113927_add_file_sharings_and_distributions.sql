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
