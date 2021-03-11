DO $$ BEGIN
  ASSERT
    (SELECT count(*) = 0 FROM forms),
    'Manual migration is required some when forms are already stored';
END $$;

ALTER TABLE forms DROP COLUMN condition;
ALTER TABLE forms DROP COLUMN unspecified_query;
ALTER TABLE forms DROP COLUMN general_query;
ALTER TABLE forms DROP COLUMN stage_query;
ALTER TABLE forms DROP COLUMN cooking_query;
ALTER TABLE forms DROP COLUMN food_query;
ALTER TABLE forms DROP COLUMN needs_sync;

DROP FUNCTION form_query_match(bit(16), integer);

CREATE TABLE form_project_query_conjunctions (
    form_id uuid NOT NULL REFERENCES forms ON DELETE RESTRICT,
    category project_category,
    attributes integer NOT NULL
);

CREATE INDEX form_project_query_conjunctions_form_id_idx
    ON form_project_query_conjunctions ( form_id );
