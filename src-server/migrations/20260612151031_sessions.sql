-- Add migration script here
CREATE TABLE sessions
(
    id    INT  NOT NULL PRIMARY KEY,
    token TEXT NOT NULL
);