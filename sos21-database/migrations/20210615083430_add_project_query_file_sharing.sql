ALTER TYPE file_sharing_scope ADD VALUE 'project_query';
COMMIT; -- we need to commit current transaction before we use 'project_query' enum value

ALTER TABLE file_sharings
    ADD COLUMN project_query jsonb,
    ADD CONSTRAINT file_sharings_scope_project_query CHECK (
        (scope = 'project_query') = (project_query IS NOT NULL)
    );
