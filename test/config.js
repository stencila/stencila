const path = require('path')
const WdioScreenshot = require('wdio-screenshot-v5')
const VisualRegressionCompare = require('wdio-novus-visual-regression-service/compare')

// Declare configuration variables and paths for storing screenshots
const screenshotDir = path.join(__dirname, 'screenshots')

const env = {
  baseUrl: process.env.BASE_URL || 'http://localhost:3000',
  staticDir: path.join(__dirname, 'examples'),
  screenshotDir,
  errors: path.join(screenshotDir, 'error'),
  examples: path.resolve(__dirname, '..', 'src', 'examples'),
  reference: path.join(screenshotDir, 'reference'),
  local: path.join(screenshotDir, 'local'),
  diff: path.join(screenshotDir, 'diff')
}

// Read browser to be tested with from the environment, falling back to Crhome
const testBrowser = process.env.TEST_BROWSER
  ? process.env.TEST_BROWSER
  : 'chrome'

// Test runner services
// Services take over a specific job you don't want to take care of. They enhance
// your test setup with almost no effort. Unlike plugins, they don't add new
// commands. Instead, they hook themselves up into the test process.
// https://webdriver.io/docs/options.html#services
const baseServices = [
  'static-server',
  'novus-visual-regression',
  [WdioScreenshot]
]

const browserCapabilities = {
  chrome: {
    browserName: 'chrome'
  },
  firefox: {
    browserName: 'firefox'
  }
}

const browserServices = {
  chrome: 'chromedriver',
  firefox: 'geckodriver'
}

// When running in CI, use SauceLabs, otherwise the browser specific driver
const services = [
  ...baseServices,
  process.env.CI ? 'sauce' : browserServices[testBrowser]
]

// Standardizes screenshot name for visual regression testing
const normalizeName = (testName, browserName, width, height) => {
  const test = testName.replace(/ /g, '_').replace('.html', '')
  const browser = browserName.toLocaleLowerCase().replace(/ /g, '_')

  return `${test}_${browser}_${width}x${height}.png`
}

// Given a test page URL will extract the final part of the pathname, without the html extension
const nameFromUrl = url =>
  url
    .split('/')
    .pop()
    .replace('.html', '')

/**
 * Given a `screenshotType`, returns a function expecting a VisualRegressionCompare context
 * @see https://github.com/Jnegrier/wdio-novus-visual-regression-service#visualregressioncomparelocalcompare
 * @param {'reference' | 'local' | 'diff' } screenshotType
 */
const getScreenshotName = screenshotType => context => {
  const testName = nameFromUrl(context.meta.url)
  const browserName = context.browser.name
  const { width, height } = context.meta.viewport

  return path.join(
    env[screenshotType],
    normalizeName(testName, browserName, width, height)
  )
}

// When running on CI, don't compare images, as we'll be using Argos to compare them
const compareStrategy =
  process.env.CI === true
    ? new VisualRegressionCompare.SaveScreenshot({
        screenshotName: getScreenshotName('local')
      })
    : new VisualRegressionCompare.LocalCompare({
        referenceName: getScreenshotName('reference'),
        screenshotName: getScreenshotName('local'),
        diffName: getScreenshotName('diff'),
        misMatchTolerance: 0.01
      })

exports.config = {
  baseUrl: env.baseUrl,
  connectionRetryTimeout: 600000,
  // Define which test specs should run. The pattern is relative to the directory
  // from which `wdio` was called. Notice that, if you are calling `wdio` from an
  // NPM script (see https://docs.npmjs.com/cli/run-script) then the current working
  // directory is where your package.json resides, so `wdio` will be called from there.
  specs: ['test/**/*.test.js'],
  // ============
  // Capabilities
  // ============
  // Define your capabilities here. WebdriverIO can run multiple capabilities at the same
  // time. Depending on the number of capabilities, WebdriverIO launches several test
  // sessions. Within your capabilities you can overwrite the spec and exclude options in
  // order to group specific specs to a specific capability.
  //
  // First, you can define how many instances should be started at the same time. Let's
  // say you have 3 different capabilities (Chrome, Firefox, and Safari) and you have
  // set maxInstances to 1; wdio will spawn 3 processes. Therefore, if you have 10 spec
  // files and you set maxInstances to 10, all spec files will get tested at the same time
  // and 30 processes will get spawned. The property handles how many capabilities
  // from the same test should run tests.
  maxInstances: 10,
  // If you have trouble getting all important capabilities together, check out the
  // Sauce Labs platform configurator - a great tool to configure your capabilities:
  // https://docs.saucelabs.com/reference/platforms-configurator
  capabilities: [
    Object.assign({}, browserCapabilities[testBrowser], {
      // When using Open Sauce (https://saucelabs.com/opensauce/),
      // capabilities must be tagged as "public" for the jobs's status
      // to update (failed/passed). If omitted on Open Sauce, the job's
      // status will only be marked "Finished."
      public: true
    })
  ],
  // By default WebdriverIO commands are executed in a synchronous way using
  // the wdio-sync package. If you still want to run your tests in an async way
  // e.g. using promises you can set the sync option to false.
  sync: false,
  // Level of logging verbosity: trace | debug | info | warn | error | silent
  logLevel: 'warn',
  // Saves a screenshot to a given path if a command fails.
  screenshotPath: env.errors,
  coloredLogs: true,
  // Default timeout for all waitFor* commands.
  waitforTimeout: 10000,
  // Default timeout in milliseconds for request
  // if Selenium Grid doesn't send response
  connectionRetryTimeout: 90000,
  // Default request retries count
  connectionRetryCount: 3,
  // Initialize the browser instance with a WebdriverIO plugin. The object should have the
  // plugin name as key and the desired plugin options as properties. Make sure you have
  // the plugin installed before running any tests.
  plugins: {},
  services,
  visualRegression: {
    // https://github.com/Jnegrier/wdio-novus-visual-regression-service
    compare: compareStrategy,
    viewportChangePause: 400,
    viewports: [{ width: 320, height: 1920 }, { width: 1440, height: 1800 }],
    orientations: ['landscape', 'portrait']
  },
  // Sauce Labs configuration
  user: process.env.SAUCE_USERNAME,
  key: process.env.SAUCE_ACCESS_KEY,
  region: 'eu',
  sauceConnect: true,
  headless: false,
  // Static Server configuration
  staticServerPort: 3000,
  staticServerFolders: [{ mount: '/', path: env.staticDir }],
  // Options for selenium-standalone
  // Path where all logs from the Selenium server should be stored.
  seleniumLogs: './logs/',
  // Framework you want to run your specs with.
  framework: 'mocha',
  // Test reporter for stdout.
  reporters: ['dot'],
  // Options to be passed to Mocha.
  // See the full list at http://mochajs.org/
  mochaOpts: {
    timeout: 0
  }
}

exports.env = env
exports.normalizeName = normalizeName
