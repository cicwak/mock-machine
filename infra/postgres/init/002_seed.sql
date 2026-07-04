INSERT INTO mock_routes (method, path_pattern, name, tags)
VALUES ('GET', '/health/example', 'Example health mock', ARRAY['example'])
ON CONFLICT (method, path_pattern) DO NOTHING;

WITH route AS (
    SELECT id
    FROM mock_routes
    WHERE method = 'GET' AND path_pattern = '/health/example'
),
scenario AS (
    INSERT INTO response_scenarios (
        route_id,
        name,
        kind,
        status_code,
        response_headers,
        response_body
    )
    SELECT
        route.id,
        'success',
        'success',
        200,
        '{"content-type":"application/json"}'::jsonb,
        '{"status":"ok"}'
    FROM route
    ON CONFLICT (route_id, name) DO UPDATE
    SET
        kind = EXCLUDED.kind,
        status_code = EXCLUDED.status_code,
        response_headers = EXCLUDED.response_headers,
        response_body = EXCLUDED.response_body,
        updated_at = now()
    RETURNING id, route_id
)
UPDATE mock_routes
SET active_scenario_id = scenario.id,
    updated_at = now()
FROM scenario
WHERE mock_routes.id = scenario.route_id;
