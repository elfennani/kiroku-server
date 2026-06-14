-- Removed them since there's no need to persist these information.
DROP TABLE metadata;
DROP TABLE metadata_streams;
DROP TABLE metadata_chapters;

-- Recreate episode_queue to not rely on episodes table.
DELETE
FROM episode_queue_temp_files;

DROP TABLE episode_queue;

CREATE TABLE episode_queue
(
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    media_id       INT  NOT NULL,
    episode_number REAL NOT NULL,
    file_path      TEXT NOT NULL,
    output_path    TEXT NOT NULL,
    step           TEXT NOT NULL CHECK (
        step IN ('IN_QUEUE',
                 'PROCESSING_1080P',
                 'PROCESSING_720P',
                 'PROCESSING_AUDIO',
                 'PROCESSING_SUBTITLES',
                 'PACKAGING',
                 'DONE')
        ),
    progress       REAL,
    created_at     INT  NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     INT  NOT NULL DEFAULT CURRENT_TIMESTAMP
);
