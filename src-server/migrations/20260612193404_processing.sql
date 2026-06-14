-- Add migration script here
CREATE TABLE episode_queue
(
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    episode_id  TEXT NOT NULL REFERENCES episode (id),
    file_path   TEXT NOT NULL,
    output_path TEXT NOT NULL,
    step        TEXT NOT NULL CHECK (
        step IN ('IN_QUEUE',
                 'PROCESSING_1080P',
                 'PROCESSING_720P',
                 'PROCESSING_AUDIO',
                 'PROCESSING_SUBTITLES',
                 'PACKAGING',
                 'DONE')
        ),
    progress    REAL,
    created_at  INT  NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  INT  NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE episode_queue_temp_files
(
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    episode_queue_id INT  NOT NULL REFERENCES episode_queue (id),
    file_path        TEXT NOT NULL
);