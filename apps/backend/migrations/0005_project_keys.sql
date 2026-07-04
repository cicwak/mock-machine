ALTER TABLE projects
    ADD COLUMN IF NOT EXISTS key TEXT;

UPDATE projects
SET key = lower(substr(replace(id::text, '-', ''), 1, 8))
WHERE key IS NULL;

ALTER TABLE projects
    ALTER COLUMN key SET NOT NULL;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'projects_key_key'
    ) THEN
        ALTER TABLE projects
            ADD CONSTRAINT projects_key_key UNIQUE (key);
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'projects_key_format_check'
    ) THEN
        ALTER TABLE projects
            ADD CONSTRAINT projects_key_format_check
            CHECK (key ~ '^[a-z0-9][a-z0-9-]{2,31}$');
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS projects_key_idx
    ON projects (key);
