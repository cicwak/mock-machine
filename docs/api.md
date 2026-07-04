# API Notes

The admin API is exposed under `/mockadminapi` through nginx.

## Health

```sh
curl http://localhost:8088/mockadminapi/health
```

## Unknown Requests

List captured unknown requests:

```sh
curl http://localhost:8088/mockadminapi/unknown-requests
```

Inspect a captured request:

```sh
curl http://localhost:8088/mockadminapi/unknown-requests/{id}
```

Convert a captured request into a mock route:

```sh
curl -X POST http://localhost:8088/mockadminapi/unknown-requests/{id}/convert \
  -H 'content-type: application/json' \
  -d '{"scenario":{"status_code":200,"response_body":"{\"ok\":true}","response_headers":{"content-type":"application/json"}}}'
```

## Projects

Set the optional default upstream URL for unconfigured mock requests:

```sh
curl -X PUT http://localhost:8088/mockadminapi/projects/{id}/settings \
  -H 'content-type: application/json' \
  -d '{"default_proxy_enabled":true,"default_proxy_url":"https://api.example.com"}'
```

## Routes

List configured mock routes:

```sh
curl http://localhost:8088/mockadminapi/routes
```

Create a dynamic profile with prefix URL composition:

```sh
curl -X POST http://localhost:8088/mockadminapi/routes/{id}/profiles \
  -H 'content-type: application/json' \
  -d '{"profile_kind":"dynamic","proxy_url":"https://api.example.com","proxy_url_mode":"prefix"}'
```

## Assets

Store an asset:

```sh
curl -X PUT http://localhost:8088/mockadminapi/assets/example.txt \
  -H 'content-type: text/plain' \
  --data-binary 'hello'
```

Read an asset:

```sh
curl http://localhost:8088/mockadminapi/assets/example.txt
```

Asset endpoints require S3-compatible storage configuration.
