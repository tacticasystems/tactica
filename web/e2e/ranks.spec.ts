import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";
import type { Rank } from "../src/lib/types";

async function workspace(page: Page, permissions = 16) {
  const unit = { id: "unit", slug: "test", display_name: "Test unit", member_count: 2 };
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
  const state = { permissions, failOrder: false };
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
    const body = method === "POST" || method === "PATCH" ? request.postDataJSON() : null;
    if (method !== "GET") {
      writes.push({ method, path, body });
      if (!(state.permissions & 17)) {
        await route.fulfill({ status: 403, json: { message: "Manage ranks required." } });
        return;
      }
    }
    if (path === "/auth/me") await route.fulfill({ json: { id: "user", username: "Tester" } });
    else if (path === "/auth/me/units") await route.fulfill({ json: { units: [unit] } });
    else if (path === "/units/unit") await route.fulfill({ json: unit });
    else if (path === "/units/unit/access")
      await route.fulfill({
        json: {
          member_id: "member",
          is_owner: false,
          permissions: state.permissions,
          highest_role_position: 0,
        },
      });
    else if (path === "/units/unit/ranks/order") {
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
  await page.goto("/units/unit/ranks");
  await expect(page.getByRole("heading", { name: "Major", exact: true })).toBeVisible();
  return {
    state,
    writes,
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
