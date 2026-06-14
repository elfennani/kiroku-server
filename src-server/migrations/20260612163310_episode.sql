-- Add migration script here
CREATE TABLE cached_media
(
    id       INT  NOT NULL PRIMARY KEY,
    title    TEXT NOT NULL,
    banner   TEXT,
    cover    TEXT,
    progress INT,
    total    INT,
    status   TEXT
);

CREATE TABLE episode
(
    id        TEXT NOT NULL PRIMARY KEY,
    media_id  INT  NOT NULL REFERENCES cached_media (id),
    title     TEXT,
    duration  INT,
    number    REAL NOT NULL,
    thumbnail TEXT,
    url       TEXT
);

CREATE TABLE chapters
(
    episode_id TEXT NOT NULL REFERENCES episode (id),
    start_time INT  NOT NULL,
    name       TEXT NOT NULL,

    PRIMARY KEY (episode_id, start_time)
);