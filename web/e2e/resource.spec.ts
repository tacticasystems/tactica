import { expect, test } from "@playwright/test";

const fixture = "/e2e/fixtures/resource.html";

test("generated list filters its page, paginates, views and saves records", async ({ page }) => {
  await page.goto(fixture);
  await expect(page.getByRole("row")).toHaveCount(21);
  await page.getByRole("searchbox").fill("Person 20");
  await expect(
    page.getByRole("heading", { name: "No matching people on this page" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Next", exact: true }).click();
  await expect(page.getByRole("searchbox")).toHaveValue("");
  await expect(page.getByRole("row")).toHaveCount(2);
  await page.getByRole("link", { name: "View Person 20", exact: true }).click();
  await page.getByRole("link", { name: "Edit person", exact: true }).click();
  await page.getByLabel("Name", { exact: true }).fill("Updated person");
  await page.getByRole("button", { name: "Save changes" }).click();
  await expect(page.getByRole("heading", { name: "Updated person", exact: true })).toBeVisible();
  await page.getByRole("link", { name: "Back to People" }).click();
  await page.getByRole("button", { name: "Next", exact: true }).click();
  await expect(page.getByRole("link", { name: "View Updated person", exact: true })).toBeVisible();
});

test("generated editor guards drafts and recovers failed mutations", async ({ page }) => {
  await page.goto(`${fixture}?route=/units/test/people/0/edit`);
  await page.getByLabel("Name", { exact: true }).fill("Unsaved");
  await page.getByRole("link", { name: "Cancel", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("unsaved changes");
  await page.getByRole("button", { name: "Keep editing" }).click();
  await expect(page.getByLabel("Name", { exact: true })).toHaveValue("Unsaved");
  await page.getByLabel("Name", { exact: true }).fill("fail");
  await page.getByRole("button", { name: "Save changes" }).click();
  await expect(page.getByRole("alert")).toHaveText("Save failed. Try again.");
  await expect(page.getByLabel("Name", { exact: true })).toHaveValue("fail");
  await page.getByLabel("Name", { exact: true }).fill("Saved");
  await page.getByRole("button", { name: "Save changes" }).click();
  await expect(page.getByRole("heading", { name: "Saved", exact: true })).toBeVisible();
  await page.getByRole("link", { name: "Edit person", exact: true }).click();
  await page.getByLabel("Name", { exact: true }).fill("Discard this");
  await page.getByRole("link", { name: "Back to People" }).click();
  await page.getByRole("button", { name: "Discard changes" }).click();
  await expect(page.getByRole("heading", { name: "People", exact: true })).toBeVisible();
  await expect(page.getByRole("link", { name: "View Saved", exact: true })).toBeVisible();
});

test("generated deletion requires confirmation and refreshes the collection", async ({ page }) => {
  await page.goto(`${fixture}?route=/units/test/people/0`);
  await page.getByRole("link", { name: "Delete person", exact: true }).click();
  await expect(page.getByText("This cannot be undone.", { exact: false })).toBeVisible();
  await page.getByRole("link", { name: "Cancel", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Person 0", exact: true })).toBeVisible();
  await page.getByRole("link", { name: "Delete person", exact: true }).click();
  await page.getByRole("button", { name: "Delete person", exact: true }).click();
  await expect(page.getByRole("heading", { name: "People", exact: true })).toBeVisible();
  await expect(page.getByRole("link", { name: "View Person 0", exact: true })).toHaveCount(0);
  await expect(page.getByText("20 people", { exact: true })).toBeVisible();
});

test("direct generated routes handle missing records and deny unauthorized mutations", async ({
  page,
}) => {
  await page.goto(`${fixture}?route=/units/test/people/missing`);
  await expect(page.getByRole("heading", { name: "Person not found", exact: true })).toBeVisible();
  await page.getByRole("link", { name: "Back to People" }).click();
  await page.getByRole("link", { name: "View Person 1", exact: true }).click();
  await expect(page.getByRole("link", { name: "Edit person", exact: true })).toHaveCount(0);
  await expect(page.getByRole("link", { name: "Delete person", exact: true })).toHaveCount(0);
  await page.goto(`${fixture}?route=/units/test/people/1/edit`);
  await expect(page.getByRole("heading", { name: "Editing unavailable" })).toBeVisible();
  await page.goto(`${fixture}?route=/units/test/people/1/delete`);
  await expect(page.getByRole("heading", { name: "Deletion unavailable" })).toBeVisible();
});
