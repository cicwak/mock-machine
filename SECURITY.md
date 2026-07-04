# Security Policy

## Supported Versions

Mock Machine is in early `0.x` development. Security fixes are currently targeted at the latest released version only.

| Version | Supported |
| --- | --- |
| `0.1.x` | Yes |
| `< 0.1.0` | No |

## Reporting a Vulnerability

Please do not open a public issue for a security vulnerability.

Report security issues through GitHub private vulnerability reporting when it is enabled for the repository. If private reporting is not available, contact the repository owner directly through GitHub.

Include:

- affected version or commit;
- reproduction steps;
- expected and actual impact;
- relevant logs or request examples with secrets removed;
- whether the issue is already publicly known.

## Deployment Notes

- The default credentials in `.env.example` are for local development only.
- Change all database, MinIO, and application secrets before exposing the stack to a network.
- Put Mock Machine behind authentication, network policy, or a trusted VPN before using it outside local development.
- Treat captured request bodies and headers as potentially sensitive data.
- Review stored mock responses and assets before sharing a database dump or MinIO bucket.
