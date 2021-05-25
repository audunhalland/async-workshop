CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE todo_item(
    id uuid NOT NULL PRIMARY KEY,
    description text NOT NULL,
    done bool NOT NULL
);
