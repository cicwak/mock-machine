CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TYPE route_status AS ENUM ('active', 'disabled');
CREATE TYPE scenario_kind AS ENUM ('success', 'error', 'timeout', 'custom');
CREATE TYPE profile_kind AS ENUM ('static', 'dynamic');
CREATE TYPE unknown_request_status AS ENUM ('new', 'ignored', 'converted');

CREATE TABLE mock_routes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    method TEXT NOT NULL,
    path_pattern TEXT NOT NULL,
    name TEXT NOT NULL,
    tags TEXT[] NOT NULL DEFAULT '{}',
    status route_status NOT NULL DEFAULT 'active',
    active_scenario_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT mock_routes_method_path_pattern_key UNIQUE (method, path_pattern),
    CONSTRAINT mock_routes_method_uppercase_check CHECK (method = upper(method)),
    CONSTRAINT mock_routes_reserved_admin_path_check CHECK (
        path_pattern !~ '^/mockadmin(/|$)' AND
        path_pattern !~ '^/mockadminapi(/|$)'
    )
);

CREATE TABLE response_scenarios (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    route_id UUID NOT NULL REFERENCES mock_routes(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    profile_kind profile_kind NOT NULL DEFAULT 'static',
    kind scenario_kind NOT NULL,
    proxy_url TEXT,
    status_code INTEGER NOT NULL DEFAULT 200,
    response_headers JSONB NOT NULL DEFAULT '{}'::jsonb,
    response_body TEXT,
    delay_ms INTEGER NOT NULL DEFAULT 0,
    selection_rules JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT response_scenarios_status_code_check CHECK (status_code BETWEEN 100 AND 599),
    CONSTRAINT response_scenarios_delay_ms_check CHECK (delay_ms >= 0),
    CONSTRAINT response_scenarios_dynamic_proxy_url_check CHECK (
        profile_kind <> 'dynamic' OR proxy_url ~ '^https?://'
    ),
    CONSTRAINT response_scenarios_route_name_key UNIQUE (route_id, name)
);

ALTER TABLE mock_routes
    ADD CONSTRAINT mock_routes_active_scenario_id_fkey
    FOREIGN KEY (active_scenario_id)
    REFERENCES response_scenarios(id)
    DEFERRABLE INITIALLY DEFERRED;

CREATE TABLE unknown_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    method TEXT NOT NULL,
    path TEXT NOT NULL,
    query JSONB NOT NULL DEFAULT '{}'::jsonb,
    headers JSONB NOT NULL DEFAULT '{}'::jsonb,
    body BYTEA,
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    count BIGINT NOT NULL DEFAULT 1,
    status unknown_request_status NOT NULL DEFAULT 'new',
    converted_route_id UUID REFERENCES mock_routes(id) ON DELETE SET NULL,
    CONSTRAINT unknown_requests_method_path_key UNIQUE (method, path),
    CONSTRAINT unknown_requests_method_uppercase_check CHECK (method = upper(method)),
    CONSTRAINT unknown_requests_count_check CHECK (count > 0)
);

CREATE TABLE object_assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bucket TEXT NOT NULL,
    object_key TEXT NOT NULL,
    content_type TEXT,
    size_bytes BIGINT,
    sha256 TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT object_assets_bucket_key_key UNIQUE (bucket, object_key)
);

CREATE INDEX mock_routes_path_pattern_idx ON mock_routes (path_pattern);
CREATE INDEX mock_routes_tags_idx ON mock_routes USING GIN (tags);
CREATE INDEX response_scenarios_route_id_idx ON response_scenarios (route_id);
CREATE INDEX unknown_requests_status_last_seen_idx ON unknown_requests (status, last_seen_at DESC);
