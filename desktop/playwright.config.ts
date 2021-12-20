import { PlaywrightTestConfig } from '@playwright/test'

const config: PlaywrightTestConfig = {
  testDir: './src/e2e',
  maxFailures: 2,
}

export default config
