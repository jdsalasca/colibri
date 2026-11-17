import { expect, test } from '@playwright/test';

test('counter, async resource and dynamic fragment', async ({ page }) => {
  const errors: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text());
  });
  page.on('pageerror', (error) => errors.push(error.message));

  await page.goto('/');
  await expect(page.locator('.value')).toHaveText('0');
  await expect(page).toHaveTitle('Contador 0');
  await expect(page.locator('.async')).toHaveText('42 manuales listos', {
    timeout: 5_000,
  });

  await expect(page.locator('.conditional')).toHaveCount(1);
  await page.locator('button.toggle').click();
  await expect(page.locator('.conditional')).toHaveCount(0);
  await page.locator('button.toggle').click();
  await expect(page.locator('.conditional')).toHaveCount(1);

  await expect(page.locator('.keyed-item')).toHaveCount(3);
  await page.evaluate(() => {
    const items = document.querySelectorAll<HTMLElement>('.keyed-item');
    items[1].dataset.marker = 'keep';
    items[1].querySelector<HTMLButtonElement>('button')?.focus();
  });
  await page.evaluate(() => {
    document.querySelector<HTMLButtonElement>('.reorder')?.click();
  });
  await expect(page.locator('.key-label')).toHaveText(['C', 'B', 'A']);
  await expect(page.locator('[data-marker="keep"] .key-label')).toHaveText('B');
  const focused = await page.evaluate(() => document.activeElement?.textContent);
  expect(focused).toBe('focus');

  await page.locator('button.increment').click();
  await expect(page.locator('.value')).toHaveText('1');
  await expect(page.locator('.double')).toHaveText('doble: 2');
  await expect(page).toHaveTitle('Contador 1');

  await expect(page.locator('[data-colibri-dynamic]')).toHaveCount(0);
  await expect(page.locator('div[style*="display: contents"]')).toHaveCount(0);
  expect(errors).toEqual([]);
});
