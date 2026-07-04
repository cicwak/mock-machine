CREATE TABLE IF NOT EXISTS projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT projects_name_key UNIQUE (name),
    CONSTRAINT projects_name_not_blank_check CHECK (btrim(name) <> '')
);

INSERT INTO projects (name)
VALUES ('Default')
ON CONFLICT (name) DO NOTHING;

ALTER TABLE mock_routes
    ADD COLUMN IF NOT EXISTS project_id UUID;

ALTER TABLE unknown_requests
    ADD COLUMN IF NOT EXISTS project_id UUID;

UPDATE mock_routes
SET project_id = (SELECT id FROM projects WHERE name = 'Default')
WHERE project_id IS NULL;

UPDATE unknown_requests
SET project_id = (SELECT id FROM projects WHERE name = 'Default')
WHERE project_id IS NULL;

ALTER TABLE mock_routes
    ALTER COLUMN project_id SET NOT NULL;

ALTER TABLE unknown_requests
    ALTER COLUMN project_id SET NOT NULL;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'mock_routes_project_id_fkey'
    ) THEN
        ALTER TABLE mock_routes
            ADD CONSTRAINT mock_routes_project_id_fkey
            FOREIGN KEY (project_id)
            REFERENCES projects(id)
            ON DELETE CASCADE;
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'unknown_requests_project_id_fkey'
    ) THEN
        ALTER TABLE unknown_requests
            ADD CONSTRAINT unknown_requests_project_id_fkey
            FOREIGN KEY (project_id)
            REFERENCES projects(id)
            ON DELETE CASCADE;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'mock_routes_method_path_pattern_key'
    ) THEN
        ALTER TABLE mock_routes
            DROP CONSTRAINT mock_routes_method_path_pattern_key;
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'mock_routes_project_method_path_pattern_key'
    ) THEN
        ALTER TABLE mock_routes
            ADD CONSTRAINT mock_routes_project_method_path_pattern_key
            UNIQUE (project_id, method, path_pattern);
    END IF;

    IF EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'unknown_requests_method_path_key'
    ) THEN
        ALTER TABLE unknown_requests
            DROP CONSTRAINT unknown_requests_method_path_key;
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'unknown_requests_project_method_path_key'
    ) THEN
        ALTER TABLE unknown_requests
            ADD CONSTRAINT unknown_requests_project_method_path_key
            UNIQUE (project_id, method, path);
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS mock_routes_project_path_pattern_idx
    ON mock_routes (project_id, path_pattern);

CREATE INDEX IF NOT EXISTS unknown_requests_project_status_last_seen_idx
    ON unknown_requests (project_id, status, last_seen_at DESC);
