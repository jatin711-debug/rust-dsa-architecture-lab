# Backend learning project

This is a standalone Cargo workspace for studying Axum, SQLx, PostgreSQL, Redis, caching, JWTs, tracing, and a React frontend. Its Docker Compose credentials are for local development only.

## Run locally

1. Copy `.env.example` to `.env` and replace `JWT_SECRET` with your own random value of at least 32 bytes. Set `CORS_ORIGIN` to the URL serving the frontend.
2. Start PostgreSQL and Redis with `docker compose up -d` from this directory.
3. Run `cargo test` and `cargo run` from this directory. Migrations run at startup and startup fails if they fail.
4. In `frontend/`, run `npm install` and `npm run dev` to use the UI.

For code checks, run `cargo fmt -- --check` and `cargo clippy --all-targets -- -D warnings` here.

Unit tests for hashing and token handling require no running services. API and repository integration tests with disposable PostgreSQL and Redis are future work; passing `cargo test` currently does not validate the full service.

Public registration creates customer accounts only. Admin accounts need a separate, trusted provisioning path; the API does not currently provide one.

The root workspace excludes this crate so the DSA learning loop stays quick. Its own `[workspace]` table makes direct Cargo commands work even though it sits under the root directory.
