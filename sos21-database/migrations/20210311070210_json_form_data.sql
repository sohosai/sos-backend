DO $$ BEGIN
  ASSERT
    (SELECT count(*) = 0 FROM forms),
    'Manual migration is required some when forms are already stored';
END $$;

ALTER TABLE forms DROP COLUMN items;
ALTER TABLE forms ADD COLUMN items jsonb NOT NULL;

ALTER TABLE form_answers DROP COLUMN items;
ALTER TABLE form_answers ADD COLUMN items jsonb NOT NULL;
