ALTER TABLE users
    ALTER COLUMN affiliation DROP NOT NULL,
    ADD CONSTRAINT users_affiliation_category CHECK (
        (category = 'undergraduate_student') = (affiliation IS NOT NULL)
    );
