#!/bin/sh
set -eu

attempt=1
until mc alias set local "http://minio:9000" "${MINIO_ROOT_USER}" "${MINIO_ROOT_PASSWORD}"; do
    if [ "${attempt}" -ge 30 ]; then
        echo "MinIO is not reachable after ${attempt} attempts." >&2
        exit 1
    fi

    attempt=$((attempt + 1))
    sleep 2
done

mc mb --ignore-existing "local/${MINIO_BUCKET}"
mc anonymous set none "local/${MINIO_BUCKET}"

cat > /tmp/mock-machine-readwrite.json <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:GetBucketLocation",
        "s3:ListBucket"
      ],
      "Resource": [
        "arn:aws:s3:::${MINIO_BUCKET}"
      ]
    },
    {
      "Effect": "Allow",
      "Action": [
        "s3:GetObject",
        "s3:PutObject",
        "s3:DeleteObject"
      ],
      "Resource": [
        "arn:aws:s3:::${MINIO_BUCKET}/*"
      ]
    }
  ]
}
EOF

mc admin policy create local mock-machine-readwrite /tmp/mock-machine-readwrite.json 2>/dev/null \
    || mc admin policy update local mock-machine-readwrite /tmp/mock-machine-readwrite.json

mc admin user add local "${MINIO_ACCESS_KEY}" "${MINIO_SECRET_KEY}" 2>/dev/null \
    || true

mc admin policy attach local mock-machine-readwrite --user "${MINIO_ACCESS_KEY}"

echo "MinIO bucket, private ACL, app user and read/write policy are ready."
