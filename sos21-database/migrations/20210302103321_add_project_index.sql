ALTER TABLE projects ADD COLUMN index smallint UNIQUE CHECK (index BETWEEN 0 AND 999);
UPDATE projects SET index = numbers.index
  FROM (SELECT id, (row_number() over () - 1) as index FROM projects)
    AS numbers
  WHERE projects.id = numbers.id;
ALTER TABLE projects ALTER COLUMN index SET NOT NULL;

CREATE SEQUENCE count_projects AS smallint MINVALUE 0 MAXVALUE 999;
SELECT setval('count_projects', (SELECT count(*) FROM projects), false);
