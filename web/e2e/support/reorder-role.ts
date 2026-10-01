import { expect, type Page } from "@playwright/test";

export async function reorderRole(
  page: Page,
  name: string,
  target: string,
  key: "ArrowUp" | "ArrowDown",
) {
  const handle = page.getByRole("button", { name: `Reorder “${name}”`, exact: true });
  await expect(handle).toBeEnabled();
  await handle.focus();
  await expect(handle).toBeFocused();
  await page.keyboard.press("Space");
  await expect(handle).toHaveAttribute("aria-pressed", "true");
  // dnd-kit defers its keyboard listener and measures droppable rectangles after pickup.
  await page.evaluate(
    () =>
      new Promise<void>((resolve) => {
        requestAnimationFrame(() => requestAnimationFrame(() => resolve()));
      }),
  );
  await page.keyboard.press(key);
  // Wait for collision detection to recognize the target before dropping.
  await expect(
    page.getByRole("status").filter({ hasText: `${name} moving to ${target}'s position.` }),
  ).toHaveText(`${name} moving to ${target}'s position.`);
  await page.keyboard.press("Space");
  await expect(page.getByRole("status").filter({ hasText: "Order saved." })).toBeVisible();
}
