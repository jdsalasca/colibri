import { expect, test, type Page } from '@playwright/test';

type Stats = { signals: number; subscriptions: number; observers: number; queued: number };

async function stats(page: Page): Promise<Stats> {
  return page.evaluate(() => JSON.parse((window as any).colibri.stats_json()));
}

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

test('teardown leaves no nodes, subscriptions or observers', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('.value')).toHaveText('0');

  await page.evaluate(() => (window as any).colibri.unmount_app());
  await expect(page.locator('#app > *')).toHaveCount(0);
  await page.waitForTimeout(700);
  const baseline = await stats(page);
  console.log('baseline stats', baseline);

  for (let i = 0; i < 25; i += 1) {
    await page.evaluate(() => (window as any).colibri.mount_app());
    await page.evaluate(() => (window as any).colibri.unmount_app());
  }
  await page.waitForTimeout(700);

  expect(await stats(page)).toEqual(baseline);
  await expect(page.locator('#app > *')).toHaveCount(0);
});

test('keyboard activates buttons and moves focus', async ({ page }) => {
  await page.goto('/');
  const increment = page.locator('button.increment');
  await increment.focus();
  await expect(increment).toBeFocused();

  await page.keyboard.press('Enter');
  await expect(page.locator('.value')).toHaveText('1');
  await page.keyboard.press('Space');
  await expect(page.locator('.value')).toHaveText('2');

  await page.keyboard.press('Shift+Tab');
  await expect(page.locator('button.reorder')).toBeFocused();
});

test('hydrates server-rendered DOM without losing nodes', async ({ page }) => {
  const errors: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text());
  });
  page.on('pageerror', (error) => errors.push(error.message));

  await page.goto('/hydrate.html');
  await page.evaluate(() => {
    const section = document.querySelector<HTMLElement>('#hydrated section');
    const value = document.querySelector<HTMLElement>('#hydrated p.value');
    if (section) section.dataset.section = 'ssr';
    if (value) value.dataset.marker = 'ssr';
  });

  await page.locator('#hydrated button.increment').click();
  await expect(page.locator('#hydrated p.value')).toHaveText('1');
  await expect(page.locator('#hydrated p.value')).toHaveAttribute('data-marker', 'ssr');
  await expect(page.locator('#hydrated section')).toHaveAttribute('data-section', 'ssr');
  await expect(page.locator('#hydrated > *')).toHaveCount(1);
  expect(errors).toEqual([]);
});

test('click to DOM latency stays bounded against a vanilla baseline', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('.value')).toHaveText('0');

  const result = await page.evaluate(() => {
    const runs = 1000;
    const wasmButton = document.querySelector<HTMLButtonElement>('button.increment');
    if (!wasmButton) throw new Error('missing increment button');
    for (let i = 0; i < 100; i += 1) wasmButton.click();
    const wasmStart = performance.now();
    for (let i = 0; i < runs; i += 1) wasmButton.click();
    const wasm = (performance.now() - wasmStart) / runs;

    const host = document.createElement('div');
    const vanillaButton = document.createElement('button');
    const output = document.createElement('span');
    output.textContent = '0';
    host.append(vanillaButton, output);
    document.body.append(host);
    let value = 0;
    vanillaButton.addEventListener('click', () => {
      value += 1;
      output.textContent = String(value);
    });
    for (let i = 0; i < 100; i += 1) vanillaButton.click();
    const vanillaStart = performance.now();
    for (let i = 0; i < runs; i += 1) vanillaButton.click();
    const vanilla = (performance.now() - vanillaStart) / runs;
    host.remove();

    return { wasm, vanilla };
  });

  console.log(
    `click→DOM per click: colibrí ${result.wasm.toFixed(4)} ms, vanilla ${result.vanilla.toFixed(4)} ms`,
  );
  expect(result.wasm).toBeLessThan(0.5);
  expect(result.vanilla).toBeLessThan(0.5);
});
