import { spawn, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../../", import.meta.url));
const binary = process.env.E2E_API_BINARY ?? `${root}target/debug/tactica-api`;

if (!process.env.E2E_DATABASE_URL) {
  throw new Error(
    "Set E2E_DATABASE_URL to a disposable PostgreSQL database; see web/e2e/README.md.",
  );
}

const env = {
  ...process.env,
  TACTICA_DB_URL: process.env.E2E_DATABASE_URL,
  TACTICA_LISTEN_ADDR: "127.0.0.1:18080",
  TACTICA_RUN_MIGRATIONS: "false",
  // Disposable fixtures shared with the API tests; never production credentials.
  TACTICA_JWT_KEY_PUB_PATH: `${root}crates/api/tests/support/jwt-public.pem`,
  TACTICA_JWT_KEY_PRIV_PATH: `${root}crates/api/tests/support/jwt-private.pem`,
  TACTICA_AUTH_SALT: "dGVzdC1zYWx0LW9ubHk",
};

const migration = spawnSync(binary, ["--run-migrations"], { cwd: root, env, stdio: "inherit" });
if (migration.error)
  throw new Error("Build the API first: cargo build -p tactica-api --bin tactica-api", {
    cause: migration.error,
  });
if (migration.status !== 0) process.exit(migration.status ?? 1);

const api = spawn(binary, [], { cwd: root, env, stdio: "inherit" });
for (const signal of ["SIGTERM", "SIGINT"]) {
  process.on(signal, () => api.kill(signal));
}
api.on("error", (error) => {
  console.error(error);
  process.exitCode = 1;
});
api.on("exit", (code, signal) => {
  process.exitCode = signal ? 0 : (code ?? 1);
});
