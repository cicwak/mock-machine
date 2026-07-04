ALTER TABLE projects
    ADD COLUMN IF NOT EXISTS default_proxy_enabled BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS default_proxy_url TEXT;

ALTER TABLE response_scenarios
    ADD COLUMN IF NOT EXISTS proxy_url_mode TEXT NOT NULL DEFAULT 'prefix';

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'projects_default_proxy_url_check'
    ) THEN
        ALTER TABLE projects
            ADD CONSTRAINT projects_default_proxy_url_check CHECK (
                default_proxy_url IS NULL OR default_proxy_url ~ '^https?://'
            );
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'response_scenarios_proxy_url_mode_check'
    ) THEN
        ALTER TABLE response_scenarios
            ADD CONSTRAINT response_scenarios_proxy_url_mode_check CHECK (
                proxy_url_mode IN ('static', 'prefix')
            );
    END IF;
END $$;
