import { defineConfig } from '@playwright/test';

const python = process.platform === 'win32' ? 'python' : 'python3';

export default defineConfig({
  testDir: '.',
  testMatch: 'e2e.spec.ts',
  timeout: 15_000,
  reporter: 'line',
  use: {
    baseURL: 'http://127.0.0.1:3211',
  },
  webServer: {
    command: `${python} -m http.server 3211 --bind 127.0.0.1`,
    url: 'http://127.0.0.1:3211',
    reuseExistingServer: true,
    timeout: 15_000,
  },
});
