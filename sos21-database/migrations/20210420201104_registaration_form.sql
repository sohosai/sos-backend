CREATE TABLE registration_forms (
    id uuid PRIMARY KEY,
    created_at timestamptz NOT NULL,
    author_id varchar(64) NOT NULL REFERENCES users ON DELETE RESTRICT,
    name varchar(64) NOT NULL,
    description varchar(1024) NOT NULL,
    items jsonb NOT NULL,
    query jsonb NOT NULL
);

CREATE TABLE registration_form_project_query_conjunctions (
    registration_form_id uuid NOT NULL REFERENCES registration_forms ON DELETE RESTRICT,
    category project_category,
    attributes integer NOT NULL
);

CREATE INDEX registration_form_project_query_conjunctions_form_id_idx
    ON registration_form_project_query_conjunctions ( registration_form_id );

CREATE TABLE registration_form_answers (
    id uuid PRIMARY KEY,
    created_at timestamptz NOT NULL,
    author_id varchar(64) NOT NULL REFERENCES users ON DELETE RESTRICT,
    registration_form_id uuid NOT NULL REFERENCES registration_forms ON DELETE RESTRICT,
    project_id uuid REFERENCES projects ON DELETE RESTRICT,
    pending_project_id uuid REFERENCES pending_projects ON DELETE RESTRICT,
    items jsonb NOT NULL,
    CONSTRAINT registration_form_answer_project_id CHECK ((project_id IS NULL) != (pending_project_id IS NULL)),
    UNIQUE (registration_form_id, pending_project_id)
);

ALTER TYPE file_sharing_scope ADD VALUE 'registration_form_answer';
COMMIT; -- we need to commit current transaction before we use 'registration_form_answer' enum value

ALTER TABLE file_sharings
    ADD COLUMN registration_form_answer_project_id uuid REFERENCES projects ON DELETE RESTRICT,
    ADD COLUMN registration_form_answer_pending_project_id uuid REFERENCES pending_projects ON DELETE RESTRICT,
    ADD COLUMN registration_form_answer_registration_form_id uuid REFERENCES registration_forms ON DELETE RESTRICT,
    ADD CONSTRAINT file_sharings_scope_registration_form_answer_respondent CHECK (
        (scope = 'registration_form_answer')::integer
            = num_nonnulls(registration_form_answer_project_id, registration_form_answer_pending_project_id)
    ),
    ADD CONSTRAINT file_sharings_scope_registration_form_answer_registration_form_id CHECK (
        (scope = 'registration_form_answer') = (registration_form_answer_registration_form_id IS NOT NULL)
    );
