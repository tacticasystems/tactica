import { readFileSync } from "node:fs";
import { randomUUID } from "node:crypto";
import { expect, test as base, type APIRequestContext, type Page } from "@playwright/test";
import { reorderRole } from "./support/reorder-role";

const password = "E2e-test-password-42!";
const credentials = () => ({ username: `e2e_${randomUUID().replaceAll("-", "")}`, password });

async function register(page: Page, username: string) {
  await page.goto("/register");
  await page.getByLabel("Username", { exact: true }).fill(username);
  await page.getByLabel("Email", { exact: true }).fill(`${username}@example.test`);
  await page.getByLabel("Password", { exact: true }).fill(password);
  await page.getByLabel("Confirm password", { exact: true }).fill(password);
  await page.getByRole("button", { name: "Create account", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Your units", exact: true })).toBeVisible();
}

async function signIn(page: Page, username: string) {
  await page.getByLabel("Username", { exact: true }).fill(username);
  await page.getByLabel("Password", { exact: true }).fill(password);
  await page.getByRole("button", { name: "Sign in", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Your units", exact: true })).toBeVisible();
}

async function signOut(page: Page, isMobile: boolean) {
  if (isMobile) await page.getByRole("button", { name: "Toggle navigation", exact: true }).click();
  await page.getByRole("button", { name: "Sign out", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Welcome back", exact: true })).toBeVisible();
}

async function session(page: Page) {
  return page.evaluate(() => JSON.parse(localStorage.getItem("tactica.session.v1")!));
}

async function apiGet(page: Page, request: APIRequestContext, path: string) {
  const { access_token } = await session(page);
  const response = await request.get(`/api/v1${path}`, {
    headers: { Authorization: `Bearer ${access_token}` },
  });
  expect(response.ok(), `GET ${path}: ${response.status()}`).toBeTruthy();
  return response.json();
}

async function createRole(page: Page, name: string) {
  await page.getByRole("button", { name: "New role", exact: true }).click();
  await page.getByLabel("Role name", { exact: true }).fill(name);
  await page.getByRole("button", { name: "Create role", exact: true }).click();
  await expect(page.getByRole("heading", { name, exact: true })).toBeVisible();
}

const test = base.extend<{ workspace: { username: string; unitId: string; unitName: string } }>({
  workspace: async ({ page }, use) => {
    const { username } = credentials();
    await register(page, username);
    await page.getByRole("button", { name: "Create a unit", exact: true }).click();
    const unitName = `Unit ${username}`;
    await page.getByLabel("Unit name", { exact: true }).fill(unitName);
    await page.getByLabel("Unit handle", { exact: true }).fill(username.replaceAll("_", "-"));
    await page.getByRole("button", { name: "Create unit", exact: true }).click();
    await expect(page.getByRole("heading", { name: "Personnel", exact: true })).toBeVisible();
    await expect(page.getByRole("table")).toBeVisible();
    const unitId = new URL(page.url()).pathname.split("/")[2];
    await use({ username, unitId, unitName });
  },
});

test("registration, unit creation, profile and sign-in survive a new session", async ({
  page,
  request,
  workspace,
  isMobile,
}) => {
  await expect(page.getByRole("row").filter({ hasText: workspace.username })).toBeVisible();
  await page.goto(`/units/${workspace.unitId}/profile`);
  await expect(page.getByLabel("Display name")).toHaveValue(workspace.unitName);
  await expect
    .poll(() => page.evaluate(() => document.documentElement.scrollWidth))
    .toBeLessThanOrEqual(page.viewportSize()!.width);
  const before = await session(page);
  await signOut(page, isMobile);
  const revoked = await request.post("/api/v1/auth/refresh", {
    data: { refresh_token: before.refresh_token },
  });
  expect(revoked.status()).toBe(401);

  await signIn(page, workspace.username);
  await page
    .getByRole("link")
    .filter({ has: page.getByRole("heading", { name: workspace.unitName, exact: true }) })
    .click();
  await expect(page.getByRole("heading", { name: "Personnel", exact: true })).toBeVisible();
  await page.reload();
  await expect(page.getByRole("row").filter({ hasText: workspace.username })).toBeVisible();
});

test("role edits persist and built-in roles remain protected", async ({
  page,
  request,
  workspace,
}) => {
  await page.goto(`/units/${workspace.unitId}/roles`);
  await createRole(page, "Medic");
  await page.getByLabel("Description", { exact: false }).fill("Treats the unit");
  await page.getByRole("checkbox", { name: /^Assign roles/ }).check();
  await page.getByRole("button", { name: "Save changes", exact: true }).click();
  await expect(page.getByRole("status").filter({ hasText: "Role saved." })).toBeVisible();
  await page.reload();
  await expect(page.getByLabel("Description", { exact: false })).toHaveValue("Treats the unit");
  await expect(page.getByRole("checkbox", { name: /^Assign roles/ })).toBeChecked();
  const { roles } = await apiGet(page, request, `/units/${workspace.unitId}/roles`);
  expect(
    roles.find((role: { display_name: string }) => role.display_name === "Medic"),
  ).toMatchObject({ permissions: 8, description: "Treats the unit" });

  await page.getByRole("button", { name: /Administrator Built-in administrator/ }).click();
  await expect(page.getByRole("checkbox").first()).toBeDisabled();
  await expect(page.getByRole("button", { name: "Delete role", exact: true })).toHaveCount(0);
  await page.getByRole("button", { name: "Everyone Applies to every member" }).click();
  await expect(page.getByLabel("Role name", { exact: true })).toHaveAttribute("readonly");
  await expect(page.getByRole("button", { name: "Delete role", exact: true })).toHaveCount(0);
});

test("assignments, removals and deletion update the persisted roster", async ({
  page,
  request,
  workspace,
}) => {
  await page.goto(`/units/${workspace.unitId}/roles`);
  await createRole(page, "Medic");
  const roleUrl = page.url();
  const roleId = new URL(roleUrl).searchParams.get("roleId")!;
  await page.getByRole("tab", { name: "Members", exact: true }).click();
  await page.getByRole("button", { name: "Add members", exact: true }).click();
  await page
    .getByRole("button", { name: `Add ${workspace.username} to Medic`, exact: true })
    .click();
  await expect(page.getByRole("status").filter({ hasText: "added to Medic" })).toBeVisible();
  await page.goto(`/units/${workspace.unitId}/personnel`);
  const pills = page.getByRole("list", { name: `${workspace.username} roles`, exact: true });
  await expect(pills.getByRole("link", { name: "Medic", exact: true })).toBeVisible();
  await page.reload();
  await pills.getByRole("link", { name: "Medic", exact: true }).click();
  await page.getByRole("tab", { name: "Members", exact: true }).click();
  await page
    .getByRole("button", { name: `Remove ${workspace.username} from Medic`, exact: true })
    .click();
  await expect(page.getByRole("status").filter({ hasText: "removed from Medic" })).toBeVisible();
  const unassigned = await apiGet(
    page,
    request,
    `/units/${workspace.unitId}/roles/${roleId}/members`,
  );
  expect(unassigned.member_ids).toEqual([]);

  await page.getByRole("button", { name: "Add members", exact: true }).click();
  await page
    .getByRole("button", { name: `Add ${workspace.username} to Medic`, exact: true })
    .click();
  await expect(page.getByRole("status").filter({ hasText: "added to Medic" })).toBeVisible();
  await page.getByRole("tab", { name: "Permissions", exact: true }).click();
  await page.getByRole("button", { name: "Delete role", exact: true }).click();
  await page
    .getByRole("alertdialog")
    .getByRole("button", { name: "Delete role", exact: true })
    .click();
  await expect(page.getByRole("alertdialog")).toBeHidden();
  await page.goto(`/units/${workspace.unitId}/personnel`);
  await expect(page.getByRole("table")).toBeVisible();
  await expect(pills.getByRole("link", { name: "Medic", exact: true })).toHaveCount(0);
  const { members } = await apiGet(page, request, `/units/${workspace.unitId}/members`);
  expect(members[0].role_ids).not.toContain(roleId);
});

test("keyboard reordering persists through the API's opposite sort order", async ({
  page,
  request,
  workspace,
  isMobile,
}) => {
  test.skip(isMobile, "Keyboard drag-and-drop uses the desktop interaction.");
  await page.goto(`/units/${workspace.unitId}/roles`);
  await createRole(page, "First role");
  await createRole(page, "Second role");
  await reorderRole(page, "Second role", "First role", "ArrowUp");
  await page.reload();
  await expect(page.locator(".role-row strong")).toHaveText([
    "Administrator",
    "Second role",
    "First role",
    "Everyone",
  ]);
  const { roles } = await apiGet(page, request, `/units/${workspace.unitId}/roles`);
  expect(roles.map((role: { display_name: string }) => role.display_name)).toEqual([
    "Administrator",
    "Second role",
    "First role",
    "Everyone",
  ]);
});

test("another account cannot read or edit a unit's roster and roles", async ({
  page,
  request,
  workspace,
  isMobile,
}) => {
  await signOut(page, isMobile);
  const outsider = credentials();
  await register(page, outsider.username);
  await page.goto(`/units/${workspace.unitId}/personnel`);
  await expect(
    page.getByRole("heading", { name: "This unit is for its members", exact: true }),
  ).toBeVisible();
  await expect(page.getByRole("table")).toHaveCount(0);
  await page.goto(`/units/${workspace.unitId}/roles`);
  await expect(
    page.getByRole("heading", { name: "This unit is for its members", exact: true }),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: "New role", exact: true })).toHaveCount(0);
  const { access_token } = await session(page);
  const response = await request.post(`/api/v1/units/${workspace.unitId}/roles`, {
    headers: { Authorization: `Bearer ${access_token}` },
    data: { display_name: "Forbidden", permissions: 0 },
  });
  expect(response.status()).toBe(403);
});

test("an expired client session refreshes once and stays signed in", async ({
  page,
  request,
  workspace,
}) => {
  const before = await session(page);
  await page.evaluate(() => {
    const key = "tactica.session.v1";
    const value = JSON.parse(localStorage.getItem(key)!);
    localStorage.setItem(key, JSON.stringify({ ...value, expires_at: 0 }));
  });
  const refreshed = page.waitForResponse(
    (response) =>
      response.url().endsWith("/api/v1/auth/refresh") && response.request().method() === "POST",
  );
  await page.reload();
  expect((await refreshed).status()).toBe(200);
  await expect(page.getByRole("row").filter({ hasText: workspace.username })).toBeVisible();
  const after = await session(page);
  expect(after.refresh_token).not.toBe(before.refresh_token);
  expect(after.session_id).toBe(before.session_id);
  expect(after.expires_at).toBeGreaterThan(Date.now());
  await apiGet(page, request, "/auth/me");
});

test("unit icon uploads through the profile control and survives reload", async ({
  page,
  request,
  workspace,
}) => {
  await page.goto(`/units/${workspace.unitId}/profile`);
  const input = page.getByLabel("Unit icon", { exact: true });
  const submit = page.getByRole("button", { name: "Upload icon", exact: true });
  await expect(input).toBeVisible();
  await expect(submit).toBeDisabled();
  await input.setInputFiles({
    name: "briefing.txt",
    mimeType: "text/plain",
    buffer: Buffer.from("not an image"),
  });
  await expect(page.getByRole("alert")).toHaveText("Choose a PNG, JPEG or WebP image.");
  await expect(submit).toBeDisabled();
  await input.setInputFiles({
    name: "oversized.png",
    mimeType: "image/png",
    buffer: Buffer.alloc(2 * 1024 * 1024 + 1),
  });
  await expect(page.getByRole("alert")).toContainText("too large");
  await expect(submit).toBeDisabled();

  const iconPath = new URL("../public/tactica-logo.png", import.meta.url).pathname;
  await input.setInputFiles(iconPath);
  await expect(page.getByRole("alert")).toHaveCount(0);
  await submit.click();
  await expect(page.getByRole("status")).toHaveText("Unit icon updated.");
  const saved = await apiGet(page, request, `/units/${workspace.unitId}`);
  expect(saved.icon_url).toContain(`/units/${workspace.unitId}/icon/`);
  const icon = page.locator(".unit-profile img");
  await expect(icon).toHaveAttribute("src", new URL(saved.icon_url, page.url()).href);
  await page.reload();
  await expect(icon).toHaveAttribute("src", new URL(saved.icon_url, page.url()).href);
  await expect
    .poll(() => icon.evaluate((img: HTMLImageElement) => img.naturalWidth))
    .toBeGreaterThan(0);

  // Server-side rejection retains the saved icon and allows a corrected retry.
  await input.setInputFiles({
    name: "broken.png",
    mimeType: "image/png",
    buffer: Buffer.from("not really a PNG"),
  });
  await submit.click();
  await expect(page.getByRole("alert")).toBeVisible();
  await expect(icon).toHaveAttribute("src", new URL(saved.icon_url, page.url()).href);
  await input.setInputFiles(iconPath);
  await submit.click();
  await expect(page.getByRole("status")).toHaveText("Unit icon updated.");
  const replacement = await apiGet(page, request, `/units/${workspace.unitId}`);
  expect(replacement.icon_url).not.toBe(saved.icon_url);
});

test("unit profile edits persist and protect unsaved changes", async ({
  page,
  request,
  workspace,
  isMobile,
}) => {
  await page.goto(`/units/${workspace.unitId}/profile`);
  const save = page.getByRole("button", { name: "Save changes", exact: true });
  await expect(save).toBeDisabled();
  await page.getByLabel("Display name", { exact: true }).fill("Raven Company");
  await page.getByLabel("Unit handle", { exact: true }).fill(`raven-${randomUUID()}`);
  await page.getByLabel("Biography", { exact: true }).fill("A team for coordinated operations.");
  // Icon changes must not discard the draft or get overwritten by Save changes.
  await page
    .getByLabel("Unit icon", { exact: true })
    .setInputFiles(new URL("../public/tactica-logo.png", import.meta.url).pathname);
  await page.getByRole("button", { name: "Upload icon", exact: true }).click();
  await expect(page.getByText("Unit icon updated.", { exact: true })).toBeVisible();
  await expect(page.getByLabel("Biography", { exact: true })).toHaveValue(
    "A team for coordinated operations.",
  );
  const icon = (await apiGet(page, request, `/units/${workspace.unitId}`)).icon_url;
  await save.click();
  await expect(page.getByText("Profile saved.", { exact: true })).toBeVisible();
  const saved = await apiGet(page, request, `/units/${workspace.unitId}`);
  expect(saved.display_name).toBe("Raven Company");
  expect(saved.icon_url).toBe(icon);
  await page.reload();
  await expect(page.getByLabel("Biography", { exact: true })).toHaveValue(saved.biography);
  await page.getByLabel("Display name", { exact: true }).fill("Discard this");
  await page.getByRole("button", { name: "Cancel", exact: true }).click();
  await expect(page.getByLabel("Display name", { exact: true })).toHaveValue("Raven Company");
  await page.getByLabel("Biography", { exact: true }).fill("Unsaved draft");
  // Use a visible internal link on either viewport to exercise router blocking.
  if (isMobile) await page.getByRole("button", { name: "Toggle navigation", exact: true }).click();
  await page.locator('a[href="/units"]:visible').first().click();
  await expect(page.getByRole("alertdialog")).toBeVisible();
  await page.getByRole("button", { name: "Keep editing", exact: true }).click();
  await expect(page.getByLabel("Biography", { exact: true })).toHaveValue("Unsaved draft");
  if (isMobile) await page.getByRole("button", { name: "Toggle navigation", exact: true }).click();
  await page.locator('a[href="/units"]:visible').first().click();
  await page.getByRole("button", { name: "Discard changes", exact: true }).click();
  await expect(page).toHaveURL(/\/units$/);
});

test("banner uploads preserve profile drafts and persist after reload", async ({
  page,
  request,
  workspace,
}) => {
  await page.goto(`/units/${workspace.unitId}/profile`);
  const form = page.getByRole("form", { name: "Upload unit banner", exact: true });
  const input = form.getByLabel("Unit banner", { exact: true });
  const upload = form.getByRole("button", { name: "Upload banner", exact: true });
  await expect(upload).toBeDisabled();
  await input.setInputFiles({
    name: "large.png",
    mimeType: "image/png",
    buffer: Buffer.alloc(5 * 1024 * 1024 + 1),
  });
  await expect(form.getByRole("alert")).toContainText("too large");
  await input.setInputFiles({
    name: "broken.png",
    mimeType: "image/png",
    buffer: Buffer.from("invalid"),
  });
  await upload.click();
  await expect(form.getByRole("alert")).toBeVisible();
  await page.getByLabel("Biography", { exact: true }).fill("Keep my draft while uploading.");
  await input.setInputFiles(new URL("../public/tactica-logo.png", import.meta.url).pathname);
  await upload.click();
  await expect(form.getByRole("status")).toHaveText("Unit banner updated.");
  const saved = await apiGet(page, request, `/units/${workspace.unitId}`);
  expect(saved.banner_url).toContain(`/units/${workspace.unitId}/banner/`);
  await expect(page.getByLabel("Biography", { exact: true })).toHaveValue(
    "Keep my draft while uploading.",
  );
  await expect(page.getByLabel("Banner URL", { exact: true })).toHaveCount(0);
  await page.getByRole("button", { name: "Save changes", exact: true }).click();
  await expect(page.getByText("Profile saved.", { exact: true })).toBeVisible();
  await page.reload();
  const banner = page.getByRole("img", { name: "Unit banner", exact: true });
  await expect(banner).toHaveAttribute("src", new URL(saved.banner_url, page.url()).href);
  await expect
    .poll(() => banner.evaluate((img: HTMLImageElement) => img.naturalWidth))
    .toBeGreaterThan(0);
  expect((await apiGet(page, request, `/units/${workspace.unitId}`)).biography).toBe(
    "Keep my draft while uploading.",
  );
});

for (const kind of ["icon", "banner"] as const) {
  test(`remove unit ${kind} confirms deletion and preserves drafts`, async ({
    page,
    request,
    workspace,
  }) => {
    await page.goto(`/units/${workspace.unitId}/profile`);
    const form = page.getByRole("form", { name: `Upload unit ${kind}`, exact: true });
    // Some OS/browser combinations provide no MIME type, even for valid images.
    await form.getByLabel(`Unit ${kind}`, { exact: true }).evaluate(
      (input: HTMLInputElement, bytes) => {
        const transfer = new DataTransfer();
        transfer.items.add(new File([new Uint8Array(bytes)], "image.png", { type: "" }));
        input.files = transfer.files;
        input.dispatchEvent(new Event("change", { bubbles: true }));
      },
      Array.from(readFileSync(new URL("../public/tactica-logo.png", import.meta.url))),
    );
    await form.getByRole("button", { name: `Upload ${kind}`, exact: true }).click();
    await expect(form.getByRole("status")).toHaveText(`Unit ${kind} updated.`);
    const url = (await apiGet(page, request, `/units/${workspace.unitId}`))[`${kind}_url`];
    await page.getByLabel("Biography", { exact: true }).fill("Keep this draft.");
    await form.getByRole("button", { name: `Remove ${kind}`, exact: true }).click();
    const dialog = page.getByRole("alertdialog");
    await dialog.getByRole("button", { name: "Cancel", exact: true }).click();
    expect((await apiGet(page, request, `/units/${workspace.unitId}`))[`${kind}_url`]).toBe(url);
    // A failed deletion keeps the dialog and image available for retry.
    await page.route("**/files/*", (route) =>
      route.request().method() === "DELETE"
        ? route.fulfill({
            status: 503,
            contentType: "application/json",
            body: JSON.stringify({ message: "Please try again." }),
          })
        : route.continue(),
    );
    await form.getByRole("button", { name: `Remove ${kind}`, exact: true }).click();
    await dialog.getByRole("button", { name: `Remove ${kind}`, exact: true }).click();
    await expect(dialog.getByRole("alert")).toHaveText("Please try again.");
    await page.unroute("**/files/*");
    await dialog.getByRole("button", { name: `Remove ${kind}`, exact: true }).click();
    await expect(dialog).toHaveCount(0);
    await expect(form.getByRole("status")).toHaveText(`Unit ${kind} removed.`);
    await expect(form.getByRole("button", { name: `Remove ${kind}`, exact: true })).toHaveCount(0);
    await expect(page.getByLabel("Biography", { exact: true })).toHaveValue("Keep this draft.");
    expect((await apiGet(page, request, `/units/${workspace.unitId}`))[`${kind}_url`]).toBeNull();
    expect((await request.get(url)).status()).toBe(404);
    await page.getByRole("button", { name: "Save changes", exact: true }).click();
    await expect(page.getByText("Profile saved.", { exact: true })).toBeVisible();
    await page.reload();
    await expect(form.getByRole("button", { name: `Remove ${kind}`, exact: true })).toHaveCount(0);
  });
}
