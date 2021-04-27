ALTER TABLE projects
    ADD CONSTRAINT projects_owner_id_subowner_id CHECK ( owner_id != subowner_id );
