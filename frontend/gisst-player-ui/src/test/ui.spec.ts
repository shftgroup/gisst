import { test, expect } from '@playwright/test';

test('has title', async ({ page }) => {
  // basic test to start things off

  await page.goto('/');

  await expect(page).toHaveTitle("Vite + TS");
});

test('Recoding Performances', async ({ page }) => {
  await page.goto('/');

  const gisstStatesList = page.locator('#gisst-replays-list');
  const initialCount = await gisstStatesList.locator('li.replay-list-item').count();

  await page.getByRole('button', { name: 'Start Performance' }).click();
  await page.getByRole('button', { name: 'Finish Performance' }).click();


  // Expects the Save State button to have added another state to the list
  await expect(gisstStatesList.locator('li.replay-list-item')).toHaveCount(initialCount+1);
});

test('Save State button', async ({ page }) => {
  await page.goto('/');

  const gisstStatesList = page.locator('#gisst-states-list');
  const initialCount = await gisstStatesList.locator('div[id*=state]').count();

  await page.getByRole('button', { name: 'Save State' }).click();

  // Expects the Save State button to have added another state to the list
  await expect(gisstStatesList.locator('div[id*=state]')).toHaveCount(initialCount+1);
});
