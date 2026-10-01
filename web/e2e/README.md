# Browser tests

The default `pnpm --dir web test:e2e` runs preview and mocked-API tests on desktop
and mobile. These cover deterministic permission and failure scenarios.

The real-API suite in `api.spec.ts` exercises registration, login/logout, unit
creation, role editing and reordering, membership assignment/removal, deletion,
access denial, and session refresh. It uses the browser UI and real HTTP requests;
it does not intercept API responses. Every test creates a unique account and unit.

## Run against PostgreSQL and the Rust API

Build the API and install Playwright's browser:

```sh
cargo build -p tactica-api --bin tactica-api
pnpm --dir web exec playwright install chromium
```

Start a disposable database (do not point these tests at a shared database):

```sh
docker run --detach --rm --name tactica-e2e-db \
  -e POSTGRES_DB=tactica_e2e -e POSTGRES_USER=e2e -e POSTGRES_PASSWORD=e2e \
  -p 127.0.0.1:15432:5432 postgres:18.6-alpine
```

Run only the real-API tests:

```sh
E2E_DATABASE_URL=postgres://e2e:e2e@127.0.0.1:15432/tactica_e2e \
  pnpm --dir web test:e2e:api
```

Or run all browser tests, including the real API:

```sh
E2E_API=1 E2E_DATABASE_URL=postgres://e2e:e2e@127.0.0.1:15432/tactica_e2e \
  pnpm --dir web test:e2e
```

Playwright applies migrations, starts the built API on `127.0.0.1:18080`, and
starts Vite on `127.0.0.1:5173` with the API proxy configured. Both ports must be
free; API-enabled runs never reuse an existing server. The API uses disposable
JWT fixtures from the Rust tests and stops with Playwright. Set `E2E_API_BINARY`
if the built binary is outside `target/debug/tactica-api`.

Stop the disposable database after testing:

```sh
docker stop tactica-e2e-db
```

CI runs both browser projects against a fresh PostgreSQL service, alongside the
preview and mocked tests. It uploads screenshots, traces, and the HTML report.
Backend changes also trigger Web CI so API/UI contract regressions are covered.
For a locally installed Chromium, set `PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH`.
