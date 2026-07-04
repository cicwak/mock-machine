INSERT INTO projects (name)
VALUES ('Default')
ON CONFLICT (name) DO NOTHING;

INSERT INTO mock_routes (project_id, method, path_pattern, name, tags)
SELECT id, 'GET', '/health/example', 'Example health mock', ARRAY['example']
FROM projects
WHERE name = 'Default'
ON CONFLICT (project_id, method, path_pattern) DO NOTHING;

WITH route AS (
    SELECT id
    FROM mock_routes
    WHERE project_id = (SELECT id FROM projects WHERE name = 'Default')
      AND method = 'GET' AND path_pattern = '/health/example'
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
