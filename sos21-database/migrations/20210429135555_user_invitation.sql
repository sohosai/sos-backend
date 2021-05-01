ALTER TABLE users ADD CONSTRAINT user_email UNIQUE ( email );

CREATE TYPE user_invitation_role AS ENUM ('administrator', 'committee_operator', 'committee');

CREATE TABLE user_invitations (
    id uuid PRIMARY KEY,
    created_at timestamptz NOT NULL,
    author_id varchar(64) NOT NULL REFERENCES users ON DELETE RESTRICT,
    email varchar(128) UNIQUE NOT NULL,
    role user_invitation_role NOT NULL
);
