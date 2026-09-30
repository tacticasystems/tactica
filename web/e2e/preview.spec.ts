import { expect, test } from "@playwright/test";

test("roster search and sidebar disclosure preserve navigation", async ({ page, isMobile }) => {
  await page.goto("/units/preview/personnel");
  await expect(page.getByRole("table")).toBeVisible();
  await expect(page.getByRole("row")).toHaveCount(7);

  await page.getByRole("searchbox").fill("Bull");
  await expect(page.getByRole("row")).toHaveCount(2);
  await page.getByRole("searchbox").fill("nobody");
  await expect(
    page.getByRole("heading", { name: "No matching members on this page" }),
  ).toBeVisible();
  await page.getByRole("searchbox").clear();
  await expect(page.getByRole("row")).toHaveCount(7);

  if (isMobile) await page.getByRole("button", { name: "Toggle navigation", exact: true }).click();

  const administration = page.getByRole("button", { name: "Administration" });
  await administration.click();
  await expect(administration).toHaveAttribute("aria-expanded", "false");
  await expect(page.getByRole("link", { name: "Profile", exact: true })).toBeHidden();
  await administration.press("Enter");
  await expect(administration).toHaveAttribute("aria-expanded", "true");
  await page.getByRole("link", { name: "Profile", exact: true }).click();
  await expect(page.getByLabel("Display name")).toHaveValue("9 Rifles");
  await expect(page.getByLabel("Display name")).toHaveAttribute("readonly");
});

test("role permissions, dirty changes, deletion and protected roles", async ({ page }) => {
  await page.goto("/units/preview/roles");
  const permissions = page.getByRole("checkbox", { name: /^Manage roles/ });
  await permissions.check();
  await page.getByRole("button", { name: "Save changes", exact: true }).click();
  await expect(page.getByRole("status").filter({ hasText: "Role saved." })).toBeVisible();
  await expect(permissions).toBeChecked();

  await page.getByLabel("Role name", { exact: true }).fill("Unsaved name");
  await page.getByRole("button", { name: "Everyone Applies to every member" }).click();
  await page.getByRole("button", { name: "Keep editing" }).click();
  await expect(page.getByLabel("Role name", { exact: true })).toHaveValue("Unsaved name");
  await page.getByRole("button", { name: "Everyone Applies to every member" }).click();
  await page.getByRole("button", { name: "Discard changes" }).click();
  await expect(page.getByLabel("Role name", { exact: true })).toHaveValue("Everyone");
  await expect(page.getByLabel("Role name", { exact: true })).toHaveAttribute("readonly");

  await page.getByRole("button", { name: "Administrator Built-in administrator" }).click();
  await expect(page.getByRole("checkbox").first()).toBeDisabled();
  await expect(page.getByRole("button", { name: "Save changes", exact: true })).toBeDisabled();

  await page.getByRole("button", { name: "New role", exact: true }).click();
  await page.getByLabel("Role name", { exact: true }).fill("Temporary role");
  await page.getByRole("button", { name: "Create role", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Temporary role", exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Delete role", exact: true }).click();
  await page.getByRole("alertdialog").getByRole("button", { name: "Cancel" }).click();
  await expect(page.getByRole("button", { name: "Delete role", exact: true })).toBeFocused();
  await page.getByRole("button", { name: "Delete role", exact: true }).click();
  await page
    .getByRole("alertdialog")
    .getByRole("button", { name: "Delete role", exact: true })
    .click();
  await expect(page.getByRole("alertdialog")).toBeHidden();
  await expect(page.getByRole("button", { name: "Temporary role Custom role" })).toHaveCount(0);
});

test("mobile navigation closes after selecting a page", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/units/preview/personnel");
  await page.getByRole("button", { name: "Toggle navigation" }).click();
  await page.getByRole("link", { name: "Roles", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Roles", exact: true })).toBeVisible();
  await expect(page.getByRole("dialog")).toBeHidden();
  await expect(page.getByRole("button", { name: "Reorder", exact: true })).toBeVisible();
});

test("keyboard role reordering saves the new order", async ({ page, isMobile }) => {
  test.skip(isMobile, "Keyboard drag-and-drop uses the desktop interaction.");
  await page.goto("/units/preview/roles");
  await page.getByRole("button", { name: "New role", exact: true }).click();
  await page.getByLabel("Role name", { exact: true }).fill("Second role");
  await page.getByRole("button", { name: "Create role", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Second role", exact: true })).toBeVisible();

  const handle = page.getByRole("button", { name: "Reorder “Second role”", exact: true });
  await handle.focus();
  await page.keyboard.press("Space");
  await page.keyboard.press("ArrowUp");
  await page.keyboard.press("Space");
  await expect(page.getByRole("status").filter({ hasText: "Order saved." })).toBeVisible();
  await expect(page.locator(".role-row strong")).toHaveText([
    "Administrator",
    "Second role",
    "Example role",
    "Everyone",
  ]);
});
