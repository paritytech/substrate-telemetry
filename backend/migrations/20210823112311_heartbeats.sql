CREATE TABLE IF NOT EXISTS heartbeats
(
    access_key TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (access_key, timestamp)
);
