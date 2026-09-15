import { test, expect } from "@playwright/test";

test("has title", async ({ page }) => {
  // basic test to start things off

  await page.goto("/");

  await expect(page).toHaveTitle("Vite + TS");
});

test("Recording Performances", async ({ page }) => {
  await page.goto("/");

  const gisstReplayList = page.locator("#gisst-replays-list");
  const initialCount = await gisstReplayList
    .locator("li.replay-list-item")
    .count();

  await page.getByRole("button", { name: "Start Performance" }).click();
  await page.getByRole("button", { name: "Finish Performance" }).click();

  // Expects recording replay to have added another replay to the list
  await expect(gisstReplayList.locator("li.replay-list-item")).toHaveCount(
    initialCount + 1,
  );
});

test("Save State button", async ({ page }) => {
  await page.goto("/");

  const gisstStatesList = page.locator("#gisst-states-list");
  const initialCount = await gisstStatesList.locator("div[id*=state]").count();

  await page.getByRole("button", { name: "Save State" }).click();

  // Expects the Save State button to have added another state to the list
  await expect(gisstStatesList.locator("div[id*=state]")).toHaveCount(
    initialCount + 1,
  );
});

test("Create Savefile button", async ({ page }) => {
  await page.goto("/");

  const gisstSaveList = page.locator("#gisst-saves");
  const initialCount = await gisstSaveList.locator("div.card-list-object").count();

  await page.getByRole("button", { name: "Create Savefile" }).click();

  // Expects the Create Savefile button to have added another save to the list
  await expect(gisstSaveList.locator("div.card-list-object")).toHaveCount(
    initialCount + 1,
  );
});
