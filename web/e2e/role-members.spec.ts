import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

async function workspace(page: Page, permissions = 8, owner = false) {
  const unit = { id: "unit", slug: "test", display_name: "Test unit", member_count: 105 };
  const roster = Array.from({ length: 105 }, (_, i) => ({
    id: `member-${i}`,
    user_id: `user-${i}`,
    unit_id: "unit",
    rank_id: "rank",
    username: `user-${i}`,
    display_name: `Person ${i}`,
    icon_url: null,
  }));
  const roles = [
    {
      id: "admin",
      display_name: "Administrator",
      kind: "administrator",
      position: 2,
      permissions: 1,
    },
    { id: "medic", display_name: "Medic", kind: "custom", position: 1, permissions: 0 },
    { id: "everyone", display_name: "Everyone", kind: "everyone", position: 0, permissions: 0 },
  ].map((role) => ({ ...role, unit_id: "unit", description: null }));
  const ids = new Set<string>();
  const writes: string[] = [];
  const state = { denied: false, failRead: false, assignedRoleId: "medic", loseResponse: false };
  await page.addInitScript(() => {
    localStorage.setItem(
      "tactica.session.v1",
      JSON.stringify({
        access_token: "test",
        refresh_token: "test",
        expires_at: Date.now() + 3600000,
        session_id: "test",
      }),
    );
  });
  await page.route("**/api/v1/**", async (route) => {
    const request = route.request();
    const url = new URL(request.url());
    const path = url.pathname.replace("/api/v1", "");
    const offset = Number(url.searchParams.get("offset") ?? 0);
    const limit = Number(url.searchParams.get("limit") ?? 100);
    let body: unknown;
    if (path === "/auth/me") body = { id: "user", username: "Tester" };
    else if (path === "/auth/me/units") body = { units: [unit].slice(offset, offset + limit) };
    else if (path === "/units/unit") body = unit;
    else if (path === "/units/unit/roles") body = { roles };
    else if (path === "/units/unit/access")
      body = {
        member_id: "member-0",
        is_owner: owner,
        permissions: state.denied ? 0 : permissions,
        highest_role_position: 2,
      };
    else if (path === "/units/unit/members")
      body = {
        members: roster.slice(offset, offset + limit).map((member) => ({
          ...member,
          role_ids: ids.has(member.id) ? [state.assignedRoleId, "everyone"] : ["everyone"],
        })),
      };
    else if (path === "/units/unit/ranks")
      body = { ranks: [{ id: "rank", unit_id: "unit", slug: "recruit", display_name: "Recruit" }] };
    else if (/\/roles\/[^/]+\/members$/.test(path)) {
      if (state.failRead) {
        await route.fulfill({ status: 500, json: { message: "Could not load assignments." } });
        return;
      }
      const roleId = path.split("/")[4];
      body = {
        member_ids: (roleId === "everyone"
          ? roster.map((member) => member.id)
          : roleId === state.assignedRoleId
            ? [...ids]
            : []
        ).slice(offset, offset + limit),
      };
    } else if (/^\/units\/unit\/roles\/[^/]+$/.test(path) && request.method() === "DELETE") {
      const roleId = path.split("/")[4];
      roles.splice(
        roles.findIndex((role) => role.id === roleId),
        1,
      );
      ids.clear();
      if (state.loseResponse) await route.abort("failed");
      else await route.fulfill({ status: 204 });
      return;
    } else if (/\/members\/[^/]+\/roles\/[^/]+$/.test(path)) {
      writes.push(`${request.method()} ${path}`);
      if (state.denied) {
        await route.fulfill({ status: 403, json: { message: "Permission revoked." } });
        return;
      }
      const memberId = path.split("/")[4];
      if (request.method() === "PUT") ids.add(memberId);
      else ids.delete(memberId);
      if (state.loseResponse) {
        await route.abort("failed");
        return;
      }
      await route.fulfill({ status: 204 });
      return;
    } else throw new Error(`Unexpected API request: ${request.method()} ${path}`);
    await route.fulfill({ json: body });
  });
  await page.goto("/units/unit/roles");
  await page.getByRole("tab", { name: "Members", exact: true }).click();
  return { writes, state, ids };
}

test("finds people beyond the first roster page, adds and removes a binding", async ({
  page,
}, testInfo) => {
  const { writes } = await workspace(page);
  await expect(page.getByText("No members have this role yet.")).toBeVisible();
  await page.getByRole("button", { name: "Add members", exact: true }).click();
  await page.getByRole("searchbox").fill("user-104");
  await page.getByRole("button", { name: "Add Person 104 to Medic", exact: true }).click();
  await expect(page.getByText("Person 104 added to Medic.", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Done adding", exact: true }).click();
  await expect(page.getByText("1 member has this role.")).toBeVisible();
  await page.screenshot({ path: testInfo.outputPath("assigned-member.png"), fullPage: true });
  await page.getByRole("button", { name: "Remove Person 104 from Medic", exact: true }).click();
  await expect(page.getByText("No members have this role yet.")).toBeVisible();
  expect(writes).toEqual([
    "PUT /units/unit/members/member-104/roles/medic",
    "DELETE /units/unit/members/member-104/roles/medic",
  ]);
});

for (const { label, permissions, owner } of [
  { label: "role manager", permissions: 4, owner: false },
  { label: "administrator", permissions: 1, owner: false },
  { label: "unit owner", permissions: 0, owner: true },
]) {
  test(`role pills link directly to the selected role for a ${label}`, async ({ page }) => {
    const { state, ids } = await workspace(page, permissions, owner);
    state.assignedRoleId = "admin";
    ids.add("member-0");
    await page.goto("/units/unit/personnel");
    const pill = page
      .getByRole("list", { name: "Person 0 roles", exact: true })
      .getByRole("link", { name: "Administrator", exact: true });
    await expect(pill).toHaveAttribute("href", "/units/unit/roles?roleId=admin");
    await pill.click();
    await expect(page.getByRole("heading", { name: "Administrator", exact: true })).toBeVisible();
    await expect(
      page.getByRole("button", { name: /Administrator Built-in administrator/ }),
    ).toHaveAttribute("aria-pressed", "true");
    await page.reload();
    await expect(page.getByRole("heading", { name: "Administrator", exact: true })).toBeVisible();
    await page.getByRole("button", { name: /Medic Custom role/ }).click();
    await expect(page).toHaveURL(/roleId=medic$/);
    await page.goBack();
    await expect(page.getByRole("heading", { name: "Administrator", exact: true })).toBeVisible();
  });
}

test("role pills stay plain for a member with only Assign roles", async ({ page }) => {
  const { ids } = await workspace(page, 8);
  ids.add("member-0");
  await page.goto("/units/unit/personnel");
  const pills = page.getByRole("list", { name: "Person 0 roles", exact: true });
  await expect(pills.getByRole("listitem")).toHaveText(["Medic"]);
  await expect(pills.getByRole("link")).toHaveCount(0);
});

test("the personnel table shows pills and refreshes after assigning and removing a role", async ({
  page,
}, testInfo) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await workspace(page);
  if (testInfo.project.name === "mobile")
    await page.getByRole("button", { name: "Toggle navigation", exact: true }).click();
  await page.getByRole("link", { name: "Personnel", exact: true }).click();
  const person = page
    .getByRole("row")
    .filter({ has: page.getByRole("list", { name: "Person 0 roles", exact: true }) });
  await expect(page.getByRole("columnheader", { name: "Roles", exact: true })).toBeVisible();
  await expect(
    person.getByRole("list", { name: "Person 0 roles", exact: true }).getByRole("listitem"),
  ).toHaveCount(0);
  if (testInfo.project.name === "mobile")
    await page.getByRole("button", { name: "Toggle navigation", exact: true }).click();
  await page.getByRole("link", { name: "Roles", exact: true }).click();
  await page.getByRole("tab", { name: "Members", exact: true }).click();
  await page.getByRole("button", { name: "Add members", exact: true }).click();
  await page.getByRole("searchbox").fill("user-0");
  await page.getByRole("button", { name: "Add Person 0 to Medic", exact: true }).click();
  await expect(page.getByText("Person 0 added to Medic.", { exact: true })).toBeVisible();
  // Client navigation exercises the cached roster rather than reloading the app.
  if (testInfo.project.name === "mobile")
    await page.getByRole("button", { name: "Toggle navigation", exact: true }).click();
  await page.getByRole("link", { name: "Personnel", exact: true }).click();
  await expect(
    person.getByRole("list", { name: "Person 0 roles", exact: true }).getByRole("listitem"),
  ).toHaveText(["Medic"]);
  if (testInfo.project.name === "mobile") await expect(page.getByRole("dialog")).not.toBeVisible();
  await page.evaluate(() => window.scrollTo(0, 0));
  await page.screenshot({ path: testInfo.outputPath("personnel-roles.png"), fullPage: true });
  if (testInfo.project.name === "mobile")
    await page.getByRole("button", { name: "Toggle navigation", exact: true }).click();
  await page.getByRole("link", { name: "Roles", exact: true }).click();
  await page.getByRole("tab", { name: "Members", exact: true }).click();
  await page.getByRole("button", { name: "Remove Person 0 from Medic", exact: true }).click();
  await expect(page.getByText("Person 0 removed from Medic.", { exact: true })).toBeVisible();
  if (testInfo.project.name === "mobile")
    await page.getByRole("button", { name: "Toggle navigation", exact: true }).click();
  await page.getByRole("link", { name: "Personnel", exact: true }).click();
  await expect(
    person.getByRole("list", { name: "Person 0 roles", exact: true }).getByRole("listitem"),
  ).toHaveCount(0);
  expect(errors).toEqual([]);
});

test("a failed membership lookup can be retried", async ({ page }) => {
  const { state } = await workspace(page);
  state.failRead = true;
  await page.reload();
  await page.getByRole("tab", { name: "Members", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("Could not load assignments.");
  state.failRead = false;
  await page.getByRole("button", { name: "Try again", exact: true }).click();
  await expect(page.getByText("No members have this role yet.")).toBeVisible();
});

test("viewing bindings does not require Manage roles or Assign roles", async ({ page }) => {
  await workspace(page, 4);
  await expect(page.getByText("No members have this role yet.")).toBeVisible();
  await expect(page.getByRole("button", { name: "Add members", exact: true })).toHaveCount(0);
  await expect(page.getByText(/requires Assign roles/)).toBeVisible();
});

test("Everyone shows the entire roster and Administrator assignments are separate from editing", async ({
  page,
}, testInfo) => {
  await workspace(page, 0, true);
  await page.getByRole("button", { name: "Everyone Applies to every member" }).click();
  await expect(page.getByText("105 members have this role.")).toBeVisible();
  await expect(page.getByRole("button", { name: "Add members", exact: true })).toHaveCount(0);
  await expect(page.getByRole("button", { name: /^Remove Person/ })).toHaveCount(0);
  await page.getByRole("searchbox").fill("user-104");
  await expect(page.getByText("Person 104", { exact: true })).toBeVisible();
  await page.screenshot({ path: testInfo.outputPath("members.png"), fullPage: true });
  await page.getByRole("button", { name: /Administrator Built-in administrator/ }).click();
  await expect(page.getByRole("button", { name: "Add members", exact: true })).toBeVisible();
});

test("failed assignments remain unassigned and refresh the caller's permissions", async ({
  page,
}) => {
  const { state } = await workspace(page);
  await page.getByRole("button", { name: "Add members", exact: true }).click();
  await page.getByRole("searchbox").fill("user-104");
  state.denied = true;
  await page.getByRole("button", { name: "Add Person 104 to Medic", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("Your permissions changed");
  await expect(page.getByRole("button", { name: "Add members", exact: true })).toHaveCount(0);
  await page.getByRole("searchbox").fill("");
  await expect(page.getByText("No members have this role yet.")).toBeVisible();
});

test("permission drafts survive switching between Permissions and Members", async ({ page }) => {
  await workspace(page, 127, true);
  await page.getByRole("tab", { name: "Permissions", exact: true }).click();
  await page.getByRole("textbox", { name: "Role name", exact: true }).fill("Field medic");
  await page.getByRole("tab", { name: "Members", exact: true }).click();
  await page.getByRole("tab", { name: "Permissions", exact: true }).click();
  await expect(page.getByRole("textbox", { name: "Role name", exact: true })).toHaveValue(
    "Field medic",
  );
  await page.getByRole("button", { name: "Everyone Applies to every member" }).click();
  await expect(
    page.getByText("You have unsaved changes. Discard them to open another role?"),
  ).toBeVisible();
});

for (const direction of ["back", "forward"] as const) {
  test(`dirty role drafts survive browser ${direction} until discarded`, async ({ page }) => {
    await workspace(page, 127, true);
    await page.getByRole("tab", { name: "Permissions", exact: true }).click();
    await page.getByRole("button", { name: "New role", exact: true }).click();
    await expect(page).toHaveURL(/roleId=new$/);
    if (direction === "forward") {
      await page.goBack();
      await expect(page.getByRole("heading", { name: "Medic", exact: true })).toBeVisible();
    }
    const name = page.getByRole("textbox", { name: "Role name", exact: true });
    await name.fill("Unsaved draft");
    await page.evaluate((direction) => window.history[direction](), direction);
    await expect(page.getByRole("alert")).toContainText("You have unsaved changes");
    await expect(name).toHaveValue("Unsaved draft");
    await page.getByRole("button", { name: "Keep editing", exact: true }).click();
    await expect(page.getByRole("alert")).toHaveCount(0);
    await expect(name).toHaveValue("Unsaved draft");
    await page.evaluate((direction) => window.history[direction](), direction);
    await page.getByRole("button", { name: "Discard changes", exact: true }).click();
    await expect(
      page.getByRole("heading", { name: direction === "back" ? "Medic" : "New role", exact: true }),
    ).toBeVisible();
    await expect(name).toHaveValue(direction === "back" ? "Medic" : "");
  });
}

test("stale role links fall back to an existing role and clear the search parameter", async ({
  page,
}) => {
  await workspace(page);
  await page.goto("/units/unit/roles?roleId=deleted-role");
  await expect(page.getByRole("heading", { name: "Medic", exact: true })).toBeVisible();
  await expect(page).toHaveURL("http://127.0.0.1:5173/units/unit/roles");
});

test("role and roster caches refresh after a committed deletion loses its response", async ({
  page,
}, testInfo) => {
  const { ids, state } = await workspace(page, 127, true);
  ids.add("member-0");
  await page.goto("/units/unit/personnel");
  const pills = page
    .getByRole("list", { name: "Person 0 roles", exact: true })
    .getByRole("listitem");
  await expect(pills).toHaveCount(1);
  if (testInfo.project.name === "mobile")
    await page.getByRole("button", { name: "Toggle navigation", exact: true }).click();
  await page.getByRole("link", { name: "Roles", exact: true }).click();
  await page.getByRole("tab", { name: "Permissions", exact: true }).click();
  state.loseResponse = true;
  await page.getByRole("button", { name: "Delete role", exact: true }).click();
  await page
    .getByRole("alertdialog")
    .getByRole("button", { name: "Delete role", exact: true })
    .click();
  await expect(page.getByRole("alertdialog")).toContainText("Check your connection");
  await page.getByRole("alertdialog").getByRole("button", { name: "Cancel", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Medic", exact: true })).toHaveCount(0);
  await expect(page.locator(".role-row").filter({ hasText: "Medic" })).toHaveCount(0);
  if (testInfo.project.name === "mobile")
    await page.getByRole("button", { name: "Toggle navigation", exact: true }).click();
  await page.getByRole("link", { name: "Personnel", exact: true }).click();
  await expect(pills).toHaveCount(0);
});

for (const initiallyAssigned of [false, true]) {
  test(`roster pills refresh after a committed ${initiallyAssigned ? "removal" : "assignment"} loses its response`, async ({
    page,
  }, testInfo) => {
    const { ids, state } = await workspace(page);
    if (initiallyAssigned) ids.add("member-0");
    await page.goto("/units/unit/personnel");
    const pills = page
      .getByRole("list", { name: "Person 0 roles", exact: true })
      .getByRole("listitem");
    await expect(pills).toHaveCount(initiallyAssigned ? 1 : 0);
    if (testInfo.project.name === "mobile")
      await page.getByRole("button", { name: "Toggle navigation", exact: true }).click();
    await page.getByRole("link", { name: "Roles", exact: true }).click();
    await page.getByRole("tab", { name: "Members", exact: true }).click();
    state.loseResponse = true;
    if (initiallyAssigned) {
      await page.getByRole("button", { name: "Remove Person 0 from Medic", exact: true }).click();
    } else {
      await page.getByRole("button", { name: "Add members", exact: true }).click();
      await page.getByRole("searchbox").fill("user-0");
      await page.getByRole("button", { name: "Add Person 0 to Medic", exact: true }).click();
    }
    await expect(page.getByRole("alert")).toContainText("Check your connection");
    await expect(
      page.getByText(initiallyAssigned ? "0 members have this role." : "1 member has this role.", {
        exact: true,
      }),
    ).toBeVisible();
    if (testInfo.project.name === "mobile")
      await page.getByRole("button", { name: "Toggle navigation", exact: true }).click();
    await page.getByRole("link", { name: "Personnel", exact: true }).click();
    await expect(pills).toHaveCount(initiallyAssigned ? 0 : 1);
  });
}
