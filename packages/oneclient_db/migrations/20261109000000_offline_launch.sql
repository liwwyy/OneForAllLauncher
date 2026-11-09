CREATE TABLE offline_launch_settings (
    cluster_id INTEGER PRIMARY KEY REFERENCES clusters(id) ON DELETE CASCADE,
    always_offline INTEGER NOT NULL DEFAULT 0 CHECK (always_offline IN (0, 1))
);
