DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'profile_kind') THEN
        CREATE TYPE profile_kind AS ENUM ('static', 'dynamic');
    END IF;
END $$;

ALTER TABLE response_scenarios
    ADD COLUMN IF NOT EXISTS profile_kind profile_kind NOT NULL DEFAULT 'static',
    ADD COLUMN IF NOT EXISTS proxy_url TEXT;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'response_scenarios_dynamic_proxy_url_check'
    ) THEN
        ALTER TABLE response_scenarios
            ADD CONSTRAINT response_scenarios_dynamic_proxy_url_check CHECK (
                profile_kind <> 'dynamic' OR proxy_url ~ '^https?://'
            );
    END IF;
END $$;
