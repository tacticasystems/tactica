import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";
import type { Rank, Role } from "../src/lib/types";

async function workspace(page: Page, permissions = 16) {
  const unit = { id: "unit", slug: "test", display_name: "Test unit", member_count: 105 };
  let ranks: Rank[] = [
    {
      id: "major",
      unit_id: "unit",
      slug: "Maj.",
      display_name: "Major",
      icon_url: null,
      description: "Unit commander",
      position: 1,
    },
    {
      id: "private",
      unit_id: "unit",
      slug: "Pvt.",
      display_name: "Private",
      icon_url: null,
      description: null,
      position: 0,
    },
  ];
  const state = {
    permissions,
    failReads: "",
    failOrder: false,
    failMembers: false,
    loseRankResponse: false,
    failMember: false,
    failMemberOnce: false,
    roles: [] as Role[],
  };
  const assignedRanks = new Map<string, string>();
  const assignedNames = new Map<string, string | null>();
  const assignedRoles = new Map<string, string[]>();
  const writes: { method: string; path: string; body: unknown }[] = [];
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
    const method = request.method();
    if (method === "GET" && state.failReads === path) {
      await route.fulfill({ status: 500, json: { message: "Background refresh failed." } });
      return;
    }
    const body =
      method === "POST" || method === "PATCH" || method === "PUT" ? request.postDataJSON() : null;
    if (method !== "GET") {
      writes.push({ method, path, body });
      const memberPatch = /^\/units\/unit\/members\/[^/]+$/.test(path) && method === "PATCH";
      const denied = memberPatch
        ? (body.rank_id !== undefined && !(state.permissions & 33)) ||
          (body.display_name !== undefined && !(state.permissions & 65)) ||
          (body.role_ids !== undefined && !(state.permissions & 9))
        : !(state.permissions & (path.endsWith("/rank") ? 33 : 17));
      if (denied) {
        await route.fulfill({ status: 403, json: { message: "Manage ranks required." } });
        return;
      }
    }
    if (path === "/auth/me") await route.fulfill({ json: { id: "user", username: "Tester" } });
    else if (path === "/auth/me/units") await route.fulfill({ json: { units: [unit] } });
    else if (path === "/units/unit") await route.fulfill({ json: unit });
    else if (path === "/units/unit/roles") await route.fulfill({ json: { roles: state.roles } });
    else if (path === "/units/unit/members") {
      if (state.failMembers) {
        await route.fulfill({ status: 500, json: { message: "Could not load members." } });
        return;
      }
      const offset = Number(url.searchParams.get("offset") ?? 0);
      const limit = Number(url.searchParams.get("limit") ?? 100);
      const members = Array.from({ length: 105 }, (_, i) => ({
        id: `member-${i}`,
        user_id: `user-${i}`,
        unit_id: "unit",
        rank_id: assignedRanks.get(`member-${i}`) ?? (i === 104 ? "major" : "private"),
        username: `user-${i}`,
        display_name: assignedNames.get(`member-${i}`) ?? `Person ${i}`,
        unit_display_name: assignedNames.get(`member-${i}`) ?? null,
        icon_url: null,
        role_ids: assignedRoles.get(`member-${i}`) ?? [],
      }));
      await route.fulfill({ json: { members: members.slice(offset, offset + limit) } });
    } else if (/^\/units\/unit\/members\/[^/]+$/.test(path) && method === "GET") {
      if (state.failMember || state.failMemberOnce) {
        state.failMemberOnce = false;
        await route.fulfill({ status: 500, json: { message: "Could not load member." } });
        return;
      }
      const id = path.split("/")[4];
      const index = Number(id.replace("member-", ""));
      if (!Number.isInteger(index) || index < 0 || index >= 105) {
        await route.fulfill({ status: 404, json: { message: "Member not found." } });
        return;
      }
      await route.fulfill({
        json: {
          id,
          user_id: `user-${index}`,
          unit_id: "unit",
          username: `user-${index}`,
          display_name: assignedNames.get(id) ?? `Person ${index}`,
          unit_display_name: assignedNames.get(id) ?? null,
          icon_url: null,
          role_ids: assignedRoles.get(id) ?? [],
          rank_id: assignedRanks.get(id) ?? (index === 104 ? "major" : "private"),
        },
      });
    } else if (/^\/units\/unit\/members\/[^/]+$/.test(path) && method === "PATCH") {
      const id = path.split("/")[4];
      if (body.rank_id !== undefined) assignedRanks.set(id, body.rank_id);
      if (body.display_name !== undefined) assignedNames.set(id, body.display_name);
      if (body.role_ids !== undefined) assignedRoles.set(id, body.role_ids);
      if (state.loseRankResponse) await route.abort("failed");
      else await route.fulfill({ status: 204 });
    } else if (path === "/units/unit/access")
      await route.fulfill({
        json: {
          member_id: "member",
          is_owner: false,
          permissions: state.permissions,
          highest_role_position: 2,
        },
      });
    else if (
      path.startsWith("/units/unit/members/") &&
      path.endsWith("/rank") &&
      method === "PUT"
    ) {
      assignedRanks.set(path.split("/")[4], body.rank_id);
      if (state.loseRankResponse) await route.abort("failed");
      else await route.fulfill({ status: 204 });
    } else if (path === "/units/unit/ranks/order") {
      if (state.failOrder) {
        await route.fulfill({ status: 400, json: { message: "The rank list changed." } });
        return;
      }
      ranks = body.rank_ids
        .map((id: string, index: number) => ({
          ...ranks.find((rank) => rank.id === id)!,
          position: index,
        }))
        .reverse();
      await route.fulfill({ json: { ranks } });
    } else if (path === "/units/unit/ranks" && method === "GET")
      await route.fulfill({ json: { ranks } });
    else if (path === "/units/unit/ranks" && method === "POST") {
      const rank = {
        ...body,
        slug: body.slug.trim(),
        id: "new-rank",
        unit_id: "unit",
        position: 0,
      };
      ranks = [...ranks.map((item) => ({ ...item, position: item.position + 1 })), rank];
      await route.fulfill({ status: 201, json: rank });
    } else if (path.startsWith("/units/unit/ranks/") && method === "PATCH") {
      const id = path.split("/").at(-1);
      const rank = { ...ranks.find((item) => item.id === id)!, ...body };
      ranks = ranks.map((item) => (item.id === id ? rank : item));
      await route.fulfill({ json: rank });
    } else if (path.startsWith("/units/unit/ranks/") && method === "DELETE") {
      const id = path.split("/").at(-1);
      if (id === "major" || id === "private") {
        await route.fulfill({
          status: 409,
          json: { message: "Rank is assigned to a member or is the initial rank." },
        });
        return;
      }
      ranks = ranks.filter((rank) => rank.id !== id);
      await route.fulfill({ status: 204 });
    } else throw new Error(`Unexpected API request: ${method} ${path}`);
  });
  await page.goto("/units/unit/admin/ranks");
  await expect(page.getByRole("heading", { name: "Major", exact: true })).toBeVisible();
  return {
    state,
    writes,
    updateMember: (id: string, rankId: string, name: string | null, roles: string[]) => {
      assignedRanks.set(id, rankId);
      assignedNames.set(id, name);
      assignedRoles.set(id, roles);
    },
    updateRank: (id: string, changes: Partial<Rank>) => {
      ranks = ranks.map((rank) => (rank.id === id ? { ...rank, ...changes } : rank));
    },
  };
}

async function refetchRanks(page: Page) {
  await page.clock.setSystemTime(new Date((await page.evaluate(() => Date.now())) + 31_000));
  const response = page.waitForResponse(
    (response) =>
      new URL(response.url()).pathname === "/api/v1/units/unit/ranks" &&
      response.request().method() === "GET",
  );
  await page.evaluate(() => {
    for (const visibilityState of ["hidden", "visible"]) {
      Object.defineProperty(document, "visibilityState", {
        configurable: true,
        value: visibilityState,
      });
      window.dispatchEvent(new Event("visibilitychange"));
    }
  });
  await response;
}

test("unchanged rank fields follow another manager's updates", async ({ page }) => {
  const { updateRank } = await workspace(page);
  updateRank("major", {
    slug: "Cmdr.",
    display_name: "Commander",
    icon_url: "https://example.com/commander.svg",
    description: "Updated responsibilities",
  });
  await refetchRanks(page);
  await expect(page.getByRole("textbox", { name: "Abbreviation", exact: true })).toHaveValue(
    "Cmdr.",
  );
  await expect(page.getByRole("textbox", { name: /Rank name/ })).toHaveValue("Commander");
  await expect(page.getByRole("textbox", { name: /Icon URL/ })).toHaveValue(
    "https://example.com/commander.svg",
  );
  await expect(page.getByRole("textbox", { name: /Description/ })).toHaveValue(
    "Updated responsibilities",
  );
  await expect(page.getByRole("button", { name: "Save rank", exact: true })).toBeDisabled();
  await page.getByRole("button", { name: /Private Pvt/ }).click();
  await expect(page.getByRole("heading", { name: "Private", exact: true })).toBeVisible();
  await expect(page.getByRole("alert")).toHaveCount(0);
});

test("refetches preserve local edits and navigation protection without overwriting other fields", async ({
  page,
}, info) => {
  const { updateRank, writes } = await workspace(page);
  await page.getByRole("textbox", { name: /Description/ }).fill("Local responsibilities");
  updateRank("major", { display_name: "Commander", description: "Remote responsibilities" });
  await refetchRanks(page);
  await expect(page.getByRole("textbox", { name: /Rank name/ })).toHaveValue("Commander");
  await expect(page.getByRole("textbox", { name: /Description/ })).toHaveValue(
    "Local responsibilities",
  );
  if (info.project.name === "mobile")
    await page.getByRole("button", { name: "Toggle navigation", exact: true }).click();
  await page.getByRole("link", { name: "Personnel", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("unsaved changes");
  await page.getByRole("button", { name: "Keep editing", exact: true }).click();
  await page.getByRole("button", { name: "Save rank", exact: true }).click();
  await expect(page.getByRole("button", { name: "Save rank", exact: true })).toBeDisabled();
  expect(writes.at(-1)).toMatchObject({
    method: "PATCH",
    body: { display_name: "Commander", description: "Local responsibilities" },
  });
});

test("matching server updates clear local edits and the unsaved-changes guard", async ({
  page,
}) => {
  const { updateRank } = await workspace(page);
  await page.getByRole("textbox", { name: /Rank name/ }).fill("Commander");
  updateRank("major", { display_name: "Commander" });
  await refetchRanks(page);
  await expect(page.getByRole("button", { name: "Save rank", exact: true })).toBeDisabled();
  updateRank("major", { display_name: "Lieutenant Colonel" });
  await refetchRanks(page);
  await expect(page.getByRole("textbox", { name: /Rank name/ })).toHaveValue("Lieutenant Colonel");
  await expect(page.getByRole("button", { name: "Save rank", exact: true })).toBeDisabled();
  await page.getByRole("button", { name: /Private Pvt/ }).click();
  await expect(page.getByRole("heading", { name: "Private", exact: true })).toBeVisible();
  await expect(page.getByRole("alert")).toHaveCount(0);
});

test("icon URLs follow the roster's HTTPS or same-origin HTTP policy", async ({ page }) => {
  const { writes } = await workspace(page);
  const icon = page.getByRole("textbox", { name: /Icon URL/ });
  const save = page.getByRole("button", { name: "Save rank", exact: true });
  await icon.fill("http://example.com/rank.svg");
  await save.click();
  await expect(icon).toHaveJSProperty(
    "validationMessage",
    "Use an HTTPS URL or an HTTP URL from this site.",
  );
  expect(writes).toEqual([]);

  for (const url of ["https://example.com/rank.svg", new URL("/rank.svg", page.url()).href]) {
    await icon.fill(url);
    await expect(icon).toHaveJSProperty("validationMessage", "");
    await save.click();
    await expect(save).toBeDisabled();
    expect(writes.at(-1)).toMatchObject({ method: "PATCH", body: { icon_url: url } });
  }
  await icon.fill("");
  await save.click();
  await expect(save).toBeDisabled();
  expect(writes.at(-1)).toMatchObject({ method: "PATCH", body: { icon_url: null } });
});

for (const permissions of [0, 4, 32]) {
  test(`members with permissions ${permissions} see ranks without edit controls`, async ({
    page,
  }) => {
    const { writes } = await workspace(page, permissions);
    await expect(page.getByRole("textbox", { name: "Abbreviation", exact: true })).toHaveValue(
      "Maj.",
    );
    const fields = page.locator(".role-editor input, .role-editor textarea");
    await expect(fields).toHaveCount(4);
    for (const field of await fields.all()) {
      await expect(field).toBeEnabled();
      await expect(field).toHaveJSProperty("readOnly", true);
      await expect(field).not.toBeEditable();
    }
    const abbreviation = page.getByRole("textbox", { name: "Abbreviation", exact: true });
    await abbreviation.focus();
    await expect(abbreviation).toBeFocused();
    await abbreviation.press("ControlOrMeta+a");
    await abbreviation.pressSequentially("Changed");
    await expect(abbreviation).toHaveValue("Maj.");
    await expect(page.getByRole("button", { name: "New rank", exact: true })).toHaveCount(0);
    await expect(page.getByRole("button", { name: /^Reorder “/ })).toHaveCount(0);
    await page.getByRole("button", { name: /Private Pvt/ }).click();
    await expect(page.getByRole("heading", { name: "Private", exact: true })).toBeVisible();
    expect(writes).toEqual([]);
  });
}

test("a rank manager creates, edits, and deletes ranks", async ({ page }) => {
  const { writes } = await workspace(page);
  await page.getByRole("button", { name: "New rank", exact: true }).click();
  await page.getByRole("textbox", { name: "Abbreviation", exact: true }).fill("  Sgt.  ");
  await page.getByRole("textbox", { name: /Rank name/ }).fill("Sergeant");
  await page.getByRole("textbox", { name: /Description/ }).fill("Section leader");
  await page.getByRole("button", { name: "Save rank", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Sergeant", exact: true })).toBeVisible();
  await expect(page.getByRole("textbox", { name: "Abbreviation", exact: true })).toHaveValue(
    "Sgt.",
  );
  await expect(page.getByRole("button", { name: "Save rank", exact: true })).toBeDisabled();
  await page.getByRole("textbox", { name: /Rank name/ }).fill("Staff Sergeant");
  await page.getByRole("textbox", { name: /Description/ }).fill("");
  await page.getByRole("button", { name: "Save rank", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Staff Sergeant", exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "Save rank", exact: true })).toBeDisabled();
  await page.getByRole("button", { name: "Delete rank", exact: true }).click();
  await page
    .getByRole("alertdialog")
    .getByRole("button", { name: "Delete rank", exact: true })
    .click();
  await expect(page.getByRole("alertdialog")).toHaveCount(0);
  await expect(page.getByRole("button", { name: /Staff Sergeant/ })).toHaveCount(0);
  expect(writes.map((write) => write.method)).toEqual(["POST", "PATCH", "DELETE"]);
  expect(writes[1].body).toMatchObject({ description: null });
});

test("keyboard reordering saves lowest-first IDs and leaves failed order unchanged", async ({
  page,
}, info) => {
  const { writes, state } = await workspace(page);
  if (info.project.name === "mobile")
    await page.getByRole("button", { name: "Reorder", exact: true }).click();
  const handle = page.getByRole("button", { name: "Reorder “Major”", exact: true });
  await handle.focus();
  await page.keyboard.press("Space");
  await expect(handle).toHaveAttribute("aria-pressed", "true");
  // KeyboardSensor attaches its arrow-key listener after activation.
  await page.evaluate(
    () =>
      new Promise<void>((resolve) =>
        requestAnimationFrame(() => requestAnimationFrame(() => resolve())),
      ),
  );
  await page.keyboard.press("ArrowDown");
  await expect(page.getByText("Major moving to Private's position.", { exact: true })).toHaveCount(
    1,
  );
  await page.keyboard.press("Space");
  await expect(page.getByText("Order saved.", { exact: true })).toBeVisible();
  const rows = page.getByRole("region", { name: "Unit ranks" }).locator(".role-row strong");
  await expect(rows).toHaveText(["Private", "Major"]);
  expect(writes[0].body).toEqual({ rank_ids: ["major", "private"] });
  state.failOrder = true;
  await handle.focus();
  await page.keyboard.press("Space");
  await expect(handle).toHaveAttribute("aria-pressed", "true");
  // KeyboardSensor attaches its arrow-key listener after activation.
  await page.evaluate(
    () =>
      new Promise<void>((resolve) =>
        requestAnimationFrame(() => requestAnimationFrame(() => resolve())),
      ),
  );
  await page.keyboard.press("ArrowUp");
  await expect(page.getByText("Major moving to Private's position.", { exact: true })).toHaveCount(
    1,
  );
  await page.keyboard.press("Space");
  await expect(page.getByRole("alert")).toContainText("The rank list changed.");
  await expect(rows).toHaveText(["Private", "Major"]);
});

test("permission revocation removes editing controls after a rejected save", async ({ page }) => {
  const { state } = await workspace(page);
  await page.getByRole("textbox", { name: /Description/ }).fill("Changed");
  state.permissions = 0;
  await page.getByRole("button", { name: "Save rank", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("Your permissions changed.");
  await expect(page.getByRole("button", { name: "New rank", exact: true })).toHaveCount(0);
  await expect(page.getByRole("textbox", { name: /Description/ })).toHaveJSProperty(
    "readOnly",
    true,
  );
  await expect(page.getByRole("textbox", { name: /Description/ })).toBeEnabled();
});

test("in-use deletion preserves the rank and unsaved drafts require discarding", async ({
  page,
}) => {
  await workspace(page);
  await page.getByRole("button", { name: "Delete rank", exact: true }).click();
  await page
    .getByRole("alertdialog")
    .getByRole("button", { name: "Delete rank", exact: true })
    .click();
  await expect(page.getByRole("alertdialog")).toContainText("Rank is assigned");
  await page.getByRole("alertdialog").getByRole("button", { name: "Cancel", exact: true }).click();
  await page.getByRole("textbox", { name: /Rank name/ }).fill("Unsaved");
  await page.getByRole("button", { name: /Private Pvt/ }).click();
  await expect(page.getByRole("alert")).toContainText("You have unsaved changes");
  await page.getByRole("textbox", { name: /Rank name/ }).fill("Unsaved");
  await page.getByRole("button", { name: "Discard changes", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Private", exact: true })).toBeVisible();
});

test("personnel ranks deep-link for ordinary members and survive reload and history", async ({
  page,
}) => {
  await workspace(page, 0);
  await page.goto("/units/unit/members");
  const link = page.getByRole("link", { name: "Private", exact: true }).first();
  await expect(link).toHaveAttribute("href", "/units/unit/ranks/private");
  await link.click();
  await expect(page.getByRole("heading", { name: "Private", exact: true })).toBeVisible();
  await page.reload();
  await expect(page.getByRole("heading", { name: "Private", exact: true })).toBeVisible();
  await page.getByRole("link", { name: "Ranks", exact: true }).last().click();
  await page.getByRole("link", { name: "Major", exact: true }).click();
  await expect(page).toHaveURL(/ranks\/major$/);
  await page.goBack();
  await expect(page.getByRole("heading", { name: "Ranks", exact: true })).toBeVisible();
  await page.goBack();
  await expect(page.getByRole("heading", { name: "Private", exact: true })).toBeVisible();
  await page.goto("/units/unit/ranks/missing");
  await expect(page.getByRole("heading", { name: "Rank not found", exact: true })).toBeVisible();
});

test("rank members include later roster pages, filter by username, and preserve drafts", async ({
  page,
}, info) => {
  await workspace(page);
  await page.getByRole("textbox", { name: /Description/ }).fill("Local draft");
  await page.getByRole("tab", { name: "Members", exact: true }).click();
  await expect(page.getByText("1 member has this rank.", { exact: true })).toBeVisible();
  await page.getByRole("searchbox").fill("user-104");
  await expect(page.getByText("Person 104", { exact: true })).toBeVisible();
  await page.screenshot({ path: info.outputPath("rank-members.png"), fullPage: true });
  await page.getByRole("searchbox").fill("user-0");
  await expect(page.getByText("No matching members. Try another name or username.")).toBeVisible();
  await page.getByRole("tab", { name: "Details", exact: true }).click();
  await expect(page.getByRole("textbox", { name: /Description/ })).toHaveValue("Local draft");
  await page.getByRole("tab", { name: "Members", exact: true }).click();
  await page.getByRole("button", { name: /Private Pvt/ }).click();
  await expect(page.getByRole("alert")).toContainText("unsaved changes");
  await page.getByRole("button", { name: "Discard changes", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Private members", exact: true })).toBeVisible();
  await expect(page.getByRole("searchbox")).toHaveValue("");
  await expect(page.getByText("104 members have this rank.")).toBeVisible();
});

test("rank members recover from a failed read and show empty ranks", async ({ page }) => {
  const { state } = await workspace(page);
  state.failMembers = true;
  await page.getByRole("tab", { name: "Members", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("Could not load members.");
  state.failMembers = false;
  await page.getByRole("button", { name: "Try again", exact: true }).click();
  await expect(page.getByText("Person 104", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "New rank", exact: true }).click();
  await expect(page.getByRole("tab", { name: "Members", exact: true })).toHaveCount(0);
  await page.getByRole("textbox", { name: "Abbreviation", exact: true }).fill("Sgt.");
  await page.getByRole("button", { name: "Save rank", exact: true }).click();
  await expect(page.getByText("No members have this rank yet.")).toBeVisible();
});

for (const permissions of [0, 16, 8]) {
  test(`member view hides editing without Assign ranks permission ${permissions}`, async ({
    page,
  }) => {
    const { writes } = await workspace(page, permissions);
    await page.goto("/units/unit/members");
    await expect(page.getByRole("button", { name: /Change rank/ })).toHaveCount(0);
    await page.getByRole("link", { name: /Person 0$/, exact: true }).click();
    await expect(page).toHaveURL(/members\/member-0$/);
    await expect(page.getByRole("heading", { name: "Person 0", exact: true })).toBeVisible();
    await expect(page.getByRole("link", { name: "Edit member", exact: true })).toHaveCount(0);
    await page.goto("/units/unit/members/member-0/edit");
    await expect(page.getByRole("combobox", { name: "Rank", exact: true })).toBeDisabled();
    await expect(page.getByRole("button", { name: "Save changes", exact: true })).toHaveCount(0);
    expect(writes).toEqual([]);
  });
}

test("member view and edit keep the roster compact and refresh rank tabs", async ({
  page,
}, info) => {
  const { writes } = await workspace(page, 32);
  await page.getByRole("tab", { name: "Members", exact: true }).click();
  await expect(page.getByText("1 member has this rank.")).toBeVisible();
  if (info.project.name === "mobile")
    await page.getByRole("button", { name: "Toggle navigation", exact: true }).click();
  await page.getByRole("link", { name: "Personnel", exact: true }).click();
  await expect(page.getByRole("button", { name: /Change rank/ })).toHaveCount(0);
  await page.screenshot({ path: info.outputPath("member-roster.png"), fullPage: true });
  await page.getByRole("link", { name: /Person 0$/, exact: true }).click();
  await expect(page.getByRole("heading", { name: "Person 0", exact: true })).toBeVisible();
  await expect(page.getByRole("link", { name: "Private", exact: true })).toBeVisible();
  await page.screenshot({ path: info.outputPath("member-view.png"), fullPage: true });
  await page.getByRole("link", { name: "Edit member", exact: true }).click();
  const form = page.getByRole("form", { name: "Edit Person 0", exact: true });
  await expect(form.getByRole("button", { name: "Save changes", exact: true })).toBeDisabled();
  await form.getByRole("combobox").click();
  await expect(page.getByRole("option")).toHaveText(["Major", "Private"]);
  await page.getByRole("option", { name: "Major", exact: true }).click();
  await page.screenshot({ path: info.outputPath("member-edit.png"), fullPage: true });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(
    true,
  );
  await form.getByRole("button", { name: "Cancel", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Person 0", exact: true })).toBeVisible();
  expect(writes).toEqual([]);
  await page.getByRole("link", { name: "Edit member", exact: true }).click();
  await expect(form.getByRole("combobox")).toHaveText("Private");
  await form.getByRole("combobox").click();
  await page.getByRole("option", { name: "Major", exact: true }).click();
  await form.getByRole("button", { name: "Save changes", exact: true }).click();
  await expect(page).toHaveURL(/members\/member-0$/);
  await page.getByRole("link", { name: "Major", exact: true }).click();
  await page.getByRole("tab", { name: "Members", exact: true }).click();
  await expect(page.getByText("2 members have this rank.")).toBeVisible();
  await expect(page.getByText("Person 0", { exact: true })).toBeVisible();
  await page.getByRole("link", { name: "Ranks", exact: true }).last().click();
  await page.getByRole("link", { name: "Private", exact: true }).click();
  await page.getByRole("tab", { name: "Members", exact: true }).click();
  await expect(page.getByText("103 members have this rank.")).toBeVisible();
  expect(writes).toEqual([
    { method: "PATCH", path: "/units/unit/members/member-0", body: { rank_id: "major" } },
  ]);
});

test("member edit recovers a lost response and handles permission revocation", async ({ page }) => {
  const { state } = await workspace(page, 32);
  await page.goto("/units/unit/members/member-0/edit");
  const form = page.getByRole("form", { name: "Edit Person 0", exact: true });
  await form.getByRole("combobox").click();
  await page.getByRole("option", { name: "Major", exact: true }).click();
  state.loseRankResponse = true;
  await form.getByRole("button", { name: "Save changes", exact: true }).click();
  await expect(page.getByRole("alert")).toBeVisible();
  await expect(form.getByRole("button", { name: "Save changes", exact: true })).toBeDisabled();
  state.loseRankResponse = false;
  state.permissions = 0;
  await form.getByRole("combobox").click();
  await page.getByRole("option", { name: "Private", exact: true }).click();
  await form.getByRole("button", { name: "Save changes", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("Your permissions changed");
  await expect(form.getByRole("combobox")).toBeDisabled();
  await expect(form.getByRole("button", { name: "Save changes", exact: true })).toHaveCount(0);
});

test("member pages reload directly and protect drafts when navigating away", async ({ page }) => {
  const { writes } = await workspace(page, 32);
  await page.goto("/units/unit/members/member-104");
  await expect(page.getByRole("heading", { name: "Person 104", exact: true })).toBeVisible();
  await page.reload();
  await expect(page.getByRole("link", { name: "Major", exact: true })).toBeVisible();
  await page.getByRole("link", { name: "Edit member", exact: true }).click();
  await page.reload();
  await page.getByRole("combobox").click();
  await page.getByRole("option", { name: "Private", exact: true }).click();
  await page.getByRole("link", { name: "View member", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("unsaved changes");
  await page.getByRole("button", { name: "Keep editing", exact: true }).click();
  await expect(page.getByRole("combobox")).toHaveText("Private");
  await page.getByRole("link", { name: "View member", exact: true }).click();
  await page.getByRole("button", { name: "Discard changes", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Person 104", exact: true })).toBeVisible();
  await expect(page.getByRole("link", { name: "Major", exact: true })).toBeVisible();
  expect(writes).toEqual([]);
});

test("member pages handle missing members and retry failed reads", async ({ page }) => {
  const { state } = await workspace(page);
  await page.goto("/units/unit/members/missing");
  await expect(page.getByRole("heading", { name: "Member not found", exact: true })).toBeVisible();
  state.failMember = true;
  await page.goto("/units/unit/members/member-0");
  await expect(page.getByRole("alert")).toContainText("Could not load member.");
  state.failMember = false;
  await page.getByRole("button", { name: "Try again", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Person 0", exact: true })).toBeVisible();
});

test("member editor saves display name and roles together with rank", async ({ page }, info) => {
  const { state, writes } = await workspace(page, 104);
  state.roles = [
    {
      id: "admin",
      unit_id: "unit",
      display_name: "Administrator",
      kind: "administrator",
      position: 2,
      permissions: 1,
      description: null,
    },
    {
      id: "medic",
      unit_id: "unit",
      display_name: "Medic",
      kind: "custom",
      position: 1,
      permissions: 0,
      description: "Provides medical support.",
    },
    {
      id: "everyone",
      unit_id: "unit",
      display_name: "Everyone",
      kind: "everyone",
      position: 0,
      permissions: 0,
      description: null,
    },
  ];
  await page.goto("/units/unit/members/member-0/edit");
  await expect(page.getByRole("checkbox", { name: /Administrator/ })).toBeDisabled();
  await page.getByRole("textbox", { name: "Display name", exact: true }).fill("Dr. Example");
  await page.getByRole("checkbox", { name: /Medic/ }).check();
  await page.getByRole("combobox").click();
  await page.getByRole("option", { name: "Major", exact: true }).click();
  await page.screenshot({ path: info.outputPath("member-edit-name-roles.png"), fullPage: true });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(
    true,
  );
  await page.getByRole("button", { name: "Save changes", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Dr. Example", exact: true })).toBeVisible();
  await expect(page.getByRole("link", { name: "Major", exact: true })).toBeVisible();
  await expect(page.getByRole("list", { name: "Dr. Example roles" })).toContainText("Medic");
  expect(writes).toEqual([
    {
      method: "PATCH",
      path: "/units/unit/members/member-0",
      body: { display_name: "Dr. Example", rank_id: "major", role_ids: ["medic"] },
    },
  ]);
  await page.getByRole("link", { name: "Edit member", exact: true }).click();
  await page.getByRole("textbox", { name: "Display name", exact: true }).fill("");
  await page.getByRole("checkbox", { name: /Medic/ }).uncheck();
  await page.getByRole("button", { name: "Save changes", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Person 0", exact: true })).toBeVisible();
  await expect(page.getByText("No assigned roles", { exact: true })).toBeVisible();
});

test("profile manager can edit only the display name", async ({ page }) => {
  await workspace(page, 64);
  await page.goto("/units/unit/members/member-0");
  await page.getByRole("link", { name: "Edit member", exact: true }).click();
  await expect(page.getByRole("textbox", { name: "Display name" })).toBeEditable();
  await expect(page.getByRole("combobox", { name: "Rank" })).toBeDisabled();
  await page.getByRole("textbox", { name: "Display name" }).fill("New name");
  await page.getByRole("link", { name: "View member", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("unsaved changes");
  await page.getByRole("button", { name: "Keep editing", exact: true }).click();
  await expect(page.getByRole("textbox", { name: "Display name" })).toHaveValue("New name");
  await page.getByRole("button", { name: "Save changes", exact: true }).click();
  await expect(page.getByRole("heading", { name: "New name", exact: true })).toBeVisible();
});

test("rank directory searches, views members, and opens the selected admin editor", async ({
  page,
}) => {
  await workspace(page);
  await page.goto("/units/unit/ranks");
  await expect(page.getByRole("table")).toBeVisible();
  await expect(page.getByRole("button", { name: "New rank", exact: true })).toHaveCount(0);
  await page.getByRole("searchbox", { name: "Search ranks" }).fill("pvt");
  await expect(page.getByRole("link", { name: "Major", exact: true })).toHaveCount(0);
  await page.getByRole("link", { name: "Private", exact: true }).click();
  await page.getByRole("tab", { name: "Members", exact: true }).click();
  await expect(page.getByText("104 members have this rank.")).toBeVisible();
  await expect(page.getByRole("link", { name: "Person 0", exact: true })).toHaveAttribute(
    "href",
    "/units/unit/members/member-0",
  );
  await page.getByRole("tab", { name: "Details", exact: true }).click();
  await page.getByRole("link", { name: "Edit rank", exact: true }).click();
  await expect(page).toHaveURL(/admin\/ranks\?rankId=private$/);
  await expect(page.getByRole("textbox", { name: "Abbreviation", exact: true })).toHaveValue(
    "Pvt.",
  );
});

test("ordinary members browse ranks without editing and old links open rank views", async ({
  page,
}) => {
  await workspace(page, 0);
  await expect(page.getByRole("textbox", { name: "Abbreviation", exact: true })).toHaveValue(
    "Maj.",
  );
  await page.goto("/units/unit/ranks?rankId=major");
  await expect(page).toHaveURL(/ranks\/major$/);
  await expect(page.getByRole("heading", { name: "Major", exact: true })).toBeVisible();
  await expect(page.getByRole("link", { name: "Edit rank", exact: true })).toHaveCount(0);
  await expect(page.getByRole("textbox")).toHaveCount(0);
});

for (const resource of ["ranks", "access"]) {
  test(`rank draft survives failed ${resource} background refresh`, async ({ page }, info) => {
    const { state } = await workspace(page);
    await page.getByRole("textbox", { name: /Description/ }).fill("Local rank draft");
    state.failReads = `/units/unit/${resource}`;
    await refetchRanks(page);
    await expect(page.getByRole("alert")).toContainText("Background refresh failed.");
    await expect(page.getByRole("textbox", { name: /Description/ })).toHaveValue(
      "Local rank draft",
    );
    await page.screenshot({ path: info.outputPath("preserved-draft.png"), fullPage: true });
    state.failReads = "";
    await page.getByRole("button", { name: "Try again", exact: true }).click();
    await expect(page.getByRole("alert")).toHaveCount(0);
    await page.getByRole("button", { name: /Private Pvt/ }).click();
    await expect(page.getByRole("alert")).toContainText("unsaved changes");
  });
}

for (const resource of ["members/member-0", "ranks", "roles", "access"]) {
  test(`member draft survives failed ${resource} background refresh`, async ({ page }, info) => {
    const { state } = await workspace(page, 104);
    state.roles = [
      {
        id: "medic",
        unit_id: "unit",
        display_name: "Medic",
        kind: "custom",
        position: 1,
        permissions: 0,
        description: null,
      },
    ];
    await page.goto("/units/unit/members/member-0/edit");
    await page
      .getByRole("textbox", { name: "Display name", exact: true })
      .fill("Local member draft");
    await page.getByRole("combobox").click();
    await page.getByRole("option", { name: "Major", exact: true }).click();
    await page.getByRole("checkbox", { name: "Medic", exact: true }).check();
    state.failReads = `/units/unit/${resource}`;
    await refetchRanks(page);
    await expect(page.getByRole("alert")).toContainText("Background refresh failed.");
    await expect(page.getByRole("textbox", { name: "Display name", exact: true })).toHaveValue(
      "Local member draft",
    );
    await expect(page.getByRole("combobox")).toHaveText("Major");
    await expect(page.getByRole("checkbox", { name: "Medic", exact: true })).toBeChecked();
    await page.screenshot({ path: info.outputPath("preserved-draft.png"), fullPage: true });
    state.failReads = "";
    await page.getByRole("button", { name: "Try again", exact: true }).click();
    await expect(page.getByRole("alert")).toHaveCount(0);
    await page.getByRole("link", { name: "View member", exact: true }).click();
    await expect(page.getByRole("alert")).toContainText("unsaved changes");
  });
}

test("committed member drafts reconcile before later server changes", async ({ page }) => {
  const { state, updateMember } = await workspace(page, 104);
  state.roles = [
    {
      id: "medic",
      unit_id: "unit",
      display_name: "Medic",
      kind: "custom",
      position: 1,
      permissions: 0,
      description: null,
    },
  ];
  await page.goto("/units/unit/members/member-0/edit");
  await page.getByRole("textbox", { name: "Display name", exact: true }).fill("  Saved name  ");
  await page.getByRole("combobox").click();
  await page.getByRole("option", { name: "Major", exact: true }).click();
  await page.getByRole("checkbox", { name: "Medic", exact: true }).check();
  state.failMemberOnce = true;
  await page.getByRole("button", { name: "Save changes", exact: true }).click();
  await expect(page.getByRole("button", { name: "Save changes", exact: true })).toBeDisabled();
  await expect(page.getByRole("textbox", { name: "Display name", exact: true })).toHaveValue(
    "Saved name",
  );
  updateMember("member-0", "private", "Remote name", []);
  await refetchRanks(page);
  await expect(page.getByRole("textbox", { name: "Display name", exact: true })).toHaveValue(
    "Remote name",
  );
  await expect(page.getByRole("combobox")).toHaveText("Private");
  await expect(page.getByRole("checkbox", { name: "Medic", exact: true })).not.toBeChecked();
  await expect(page.getByRole("button", { name: "Save changes", exact: true })).toBeDisabled();
  await page.getByRole("textbox", { name: "Display name", exact: true }).fill("Local name");
  updateMember("member-0", "major", "Another remote name", ["medic"]);
  await refetchRanks(page);
  await expect(page.getByRole("textbox", { name: "Display name", exact: true })).toHaveValue(
    "Local name",
  );
  await expect(page.getByRole("combobox")).toHaveText("Major");
  await expect(page.getByRole("checkbox", { name: "Medic", exact: true })).toBeChecked();
  await expect(page.getByRole("button", { name: "Save changes", exact: true })).toBeEnabled();
});
