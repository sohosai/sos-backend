ALTER TYPE project_category RENAME TO project_category_old;
CREATE TYPE project_category AS ENUM('general', 'cooking_requiring_preparation_area', 'cooking', 'food', 'stage');
ALTER TABLE projects ALTER COLUMN category TYPE project_category USING category::text::project_category;
ALTER TABLE pending_projects ALTER COLUMN category TYPE project_category USING category::text::project_category;
ALTER TABLE form_project_query_conjunctions ALTER category TYPE project_category USING category::text::project_category;
ALTER TABLE registration_form_project_query_conjunctions ALTER category TYPE project_category USING category::text::project_category;
DROP TYPE project_category_old;
