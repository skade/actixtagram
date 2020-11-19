-- Add migration script here
CREATE TABLE IF NOT EXISTS uploads
(
    id          INTEGER PRIMARY KEY NOT NULL,
    filename    TEXT                NOT NULL,
    processed   BOOLEAN             NOT NULL DEFAULT 0
);