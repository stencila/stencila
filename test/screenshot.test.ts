/**
 * Run visual regression tests.
 *
 * Note that `npm run test:build` must be run before running these tests.
 * That script runs Parcel to build the `test/build` directory with all the
 * HTML, compiled CSS and JS used for visual regression testing in it.
 */

import assert from 'assert'
import fs from 'fs'
import http from 'http'
import { staticDir, baseUrl } from './wdio.config'
import { examples } from '../src/examples'
import { themes } from '../src/themes/index'

// The examples to use for visual regression tests
// It's generally better to only use small examples, as
// larger ones take up time and space
const EXAMPLES = [examples.articleKitchenSink]

// The themes to be tested. Defaults to all
const THEMES = Object.keys(themes)

// Get hostname and port for doing connectivity tests
const { hostname, port } = new URL(baseUrl)

describe('visual regressions: ', () => {
  // Test that the test/build folder has been already built
  it(`needs the build folder`, () => {
    assert.ok(
      fs.existsSync(staticDir),
      `${staticDir} does not exist. Did you run 'npm run test:build' first?`
    )
  })

  // Test that /index.html is available. We could check that
  // full paths are available e.g. ?example=...&theme=...
  // but that doesn't help since examples and themese are loaded dynamically
  it(`needs index.html`, () => {
    const req = http.get({ hostname, port, path: '/index.html' }, res => {
      assert.equal(res.statusCode, '200')
    })
    req.on('error', error => assert.fail(error))
    req.end()
  })

  describe(`runs over examples and themes: `, () => {
    EXAMPLES.forEach(example => {
      THEMES.forEach(theme => {
        const path = `?example=${example}&theme=${theme}&header=false`

        // A pseudo-test that is helpful for debugging the page
        // that the screen-shotting actually sees. To use it un-skip it.
        it.skip(`${path}: can be browsed`, async () => {
          console.log(
            `Browse for 60s before the robots 🤖 take control: ${baseUrl}${path}`
          )
          await new Promise(resolve => setTimeout(resolve, 60000))
        })

        it(`${path}: screenshots have not changed`, async () => {
          // @ts-ignore
          await browser.url(path)
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
})
