# Military Portal backend

Rust backend for the Military Portal Deployment project found in github.com/Sijma/military-portal-deployment.
The same, compiled binary can run as as one of three role-based instances: `citizen`, `officer`, or `admin`. Each service provides a `/health` endpoint.

The full application is meant to be run from the unified deployment repository with Docker Compose.

## Running one service

The binary expects PostgreSQL and SMTP configuration through environment variables:
- `DATABASE_URL`, `PGUSER`, `PGPASSWORD`, `PGDATABASE`
- `MAIL_HOST`, `MAIL_PORT`, `MAIL_FROM`

It also expects 2 Cli Arguments:
- `--service-instance <role>:<port>`, where role is one of `[citizen, officer, admin]`
- `--bind-host <ip>`

Example:
```shell
cargo run -- \
  --service-instance citizen:8081 \
  --bind-host 127.0.0.1
```

Authentication is handled by APISIX and Keycloak.
The backend trusts the identity headers inserted by APISIX and does verify JWTs, so the service ports must not be exposed externally.

Pushing a tagged commit triggers Jenkins to build and publish:
- A container image
- A native release binary used by Ansible downstream.
