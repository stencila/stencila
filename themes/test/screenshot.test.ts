/**
 * Run visual regression tests.
 *
 * Note that `npm run docs` must be run before running these tests.
 * That script generates the HTML, compiled CSS and JS used for visual regression testing.
 */

import assert from 'assert'
import fs from 'fs'
import http from 'http'
import { examples } from '../src/examples'
import { themes } from '../src/themes/index'
import { baseUrl, staticDir } from './wdio.config'

// The examples to use for visual regression tests
// It's generally better to only use small examples, as
// larger ones take up time and space
const EXAMPLES = [examples.articleKitchenSink, examples.articlePests]

// The themes to be tested. Defaults to all
const THEMES = Object.keys(themes)

// Get hostname and port for doing connectivity tests
const { hostname, port } = new URL(baseUrl)

describe('visual regressions: ', () => {
  // Test that the test/build folder has been already built
  it(`needs the build folder`, () => {
    assert.ok(
      fs.existsSync(staticDir),
      `${staticDir} does not exist. Did you run 'npm run docs' first?`
    )
  })

  // Don't run visual regression tests if the build folder doesn't exists.
  // This avoids burying the instruction to run `npm run docs` in other error reports.
  if (fs.existsSync(staticDir)) {
    // Test that /index.html is available. We could check that
    // full paths are available e.g. ?example=...&theme=...
    // but that doesn't help since examples and themese are loaded dynamically
    it(`needs index.html`, () => {
      const req = http.get({ hostname, port, path: '/index.html' }, (res) => {
        assert.equal(res.statusCode, '200')
      })
      req.on('error', (error) => assert.fail(error))
      req.end()
    })

    describe(`runs over examples and themes: `, () => {
      EXAMPLES.forEach((example) => {
        THEMES.forEach((theme) => {
          const path = `/editor?example=${example}&theme=${theme}&ui=false`

          // A pseudo-test that is helpful for debugging the page
          // that the screen-shotting actually sees. To use it un-skip it.
          it.skip(`${theme}/${example}: can be browsed`, async () => {
            console.log(
              `Browse for 60s before the robots 🤖 take control: ${baseUrl}${path}`
            )
            await new Promise((resolve) => setTimeout(resolve, 60000))
          })

          it(`${theme}/${example}: screenshots have not changed`, async () => {
            // @ts-ignore
            await browser.url(path)
            // Tell WDIO to take control of preview iframe content, instead of Theme Editor
            // @ts-ignore
            const frame = await browser.$('#preview')
            // @ts-ignore
            await browser.switchToFrame(frame)

            // Wait for page to completely load
            // @ts-ignore
            browser.waitUntil(
              async () => {
                // @ts-ignore
                const state = await browser.execute(() => document.readyState)
                return state === 'complete'
              },
              {
                timeout: 60000,
                timeoutMsg: 'The page did not load in time',
              }
            )

            // @ts-ignore
            const results = await browser.checkDocument()

            /**
             * Takes an array of visual regression comparison results, and checks if all
             * are within difference tolerances.
             */
            const allScreenshotsPass = (results: any[]) =>
              results.reduce(
                (pass: boolean, result: any) =>
                  pass && result.isWithinMisMatchTolerance === true,
                true
              )
            assert.ok(
              allScreenshotsPass(results),
              `Styles differ from current references.`
            )
          })
        })
      })
    })
  }
})
