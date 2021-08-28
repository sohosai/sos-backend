ALTER TABLE projects ADD COLUMN updated_at timestamptz;
UPDATE projects SET updated_at = created_at;
ALTER TABLE projects ALTER COLUMN updated_at SET NOT NULL;

ALTER TABLE pending_projects ADD COLUMN updated_at timestamptz;
UPDATE pending_projects SET updated_at = created_at;
ALTER TABLE pending_projects ALTER COLUMN updated_at SET NOT NULL;

ALTER TABLE registration_form_answers ADD COLUMN updated_at timestamptz;
UPDATE registration_form_answers SET updated_at = created_at;
ALTER TABLE registration_form_answers ALTER COLUMN updated_at SET NOT NULL;
