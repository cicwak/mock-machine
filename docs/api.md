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

## Routes

List configured mock routes:

```sh
curl http://localhost:8088/mockadminapi/routes
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
