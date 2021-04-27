CREATE TYPE user_assignment AS ENUM ('project_owner', 'project_subowner', 'pending_project_owner');

ALTER TABLE users
    ADD COLUMN assignment user_assignment,
    ADD COLUMN assignment_owner_project_id uuid UNIQUE REFERENCES projects ON DELETE RESTRICT,
    ADD COLUMN assignment_subowner_project_id uuid UNIQUE REFERENCES projects ON DELETE RESTRICT,
    ADD COLUMN assignment_owner_pending_project_id uuid UNIQUE REFERENCES pending_projects ON DELETE RESTRICT,
    ADD CONSTRAINT users_assignment_owner_project_id CHECK ((assignment = 'project_owner') = (assignment_owner_project_id IS NOT NULL)),
    ADD CONSTRAINT users_assignment_subowner_project_id CHECK ((assignment = 'project_subowner') = (assignment_subowner_project_id IS NOT NULL)),
    ADD CONSTRAINT users_assignment_owner_pending_project_id CHECK ((assignment = 'pending_project_owner') = (assignment_owner_pending_project_id IS NOT NULL));

DO $$ BEGIN
  ASSERT
    (SELECT
        (SELECT count(*) FROM (
            SELECT DISTINCT owner_id FROM projects
            UNION SELECT DISTINCT subowner_id FROM projects
            UNION SELECT DISTINCT author_id FROM pending_projects
        ) AS users)
        = (SELECT count(*) * 2 FROM projects) + (SELECT count(*) FROM pending_projects)
    ),
    'Manual migration is required when project owners are already duplicated';
END $$;

ALTER TABLE projects
    ADD CONSTRAINT projects_owner_id UNIQUE ( owner_id ),
    ADD CONSTRAINT projects_subowner_id UNIQUE ( subowner_id );

UPDATE users
SET
    assignment = 'project_owner',
    assignment_owner_project_id = projects.id
FROM projects
WHERE projects.owner_id = users.id;

UPDATE users
SET
    assignment = 'project_subowner',
    assignment_subowner_project_id = projects.id
FROM projects
WHERE projects.subowner_id = users.id;

ALTER TABLE pending_projects
    ADD CONSTRAINT pending_projects_author_id UNIQUE ( author_id );

UPDATE users
SET
    assignment = 'pending_project_owner',
    assignment_owner_pending_project_id = pending_projects.id
FROM pending_projects
WHERE pending_projects.author_id = users.id;

ALTER TABLE projects
    DROP COLUMN owner_id,
    DROP COLUMN subowner_id;
ALTER TABLE pending_projects
    DROP COLUMN author_id;

-- NOTE: We must have one owner/subowner for each project/pending_project entry, but that is not expressed as constraints
