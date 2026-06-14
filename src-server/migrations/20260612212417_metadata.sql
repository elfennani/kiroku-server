-- Add migration script here
CREATE TABLE IF NOT EXISTS metadata
(
    id       INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    title    TEXT    NOT NULL,
    duration INT     NOT NULL
);
CREATE TABLE IF NOT EXISTS metadata_chapters
(
    id          INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    metadata_id INTEGER NOT NULL REFERENCES metadata (id),
    title       TEXT,
    start       INT     NOT NULL,
    end         INT     NOT NULL,
    `index`     INT     NOT NULL
);
CREATE TABLE IF NOT EXISTS metadata_streams
(
    id          INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    metadata_id INTEGER NOT NULL REFERENCES metadata (id),
    title       TEXT    NOT NULL,
    language    TEXT    NOT NULL,
    `index`     INT     NOT NULL,
    type        TEXT    NOT NULL CHECK ( type in ('audio', 'subtitle') )
);