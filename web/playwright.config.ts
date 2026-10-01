import { defineConfig, devices } from "@playwright/test";

const apiEnabled = process.env.E2E_API === "1";

export default defineConfig({
  testDir: "./e2e",
  testIgnore: apiEnabled ? [] : ["**/api.spec.ts"],
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: process.env.CI ? [["list"], ["html", { open: "never" }]] : "list",
  use: {
    baseURL: "http://127.0.0.1:5173",
    screenshot: "on",
    trace: "on-first-retry",
    launchOptions: { executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH },
  },
  projects: [
    { name: "desktop", use: { ...devices["Desktop Chrome"] } },
    { name: "mobile", use: { ...devices["iPhone 13"], defaultBrowserType: "chromium" } },
  ],
  webServer: [
    ...(apiEnabled
      ? [
          {
            command: "node e2e/start-api.mjs",
            url: "http://127.0.0.1:18080/healthz",
            reuseExistingServer: false,
            timeout: 60_000,
            gracefulShutdown: { signal: "SIGTERM" as const, timeout: 5_000 },
          },
        ]
      : []),
    {
      command: "pnpm dev",
      url: "http://127.0.0.1:5173",
      reuseExistingServer: !process.env.CI && !apiEnabled,
      env: apiEnabled ? { TACTICA_API_PROXY: "http://127.0.0.1:18080" } : {},
    },
  ],
});
