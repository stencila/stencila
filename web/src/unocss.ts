import { presetIcons } from '@unocss/preset-icons/browser'
import { presetWind4 } from '@unocss/preset-wind4'
import initUnocssRuntime from '@unocss/runtime'

import { GlideEvents } from './site/glide/events'

let unoInitialized = false
let glideHooked = false

/**
 * Styled content wrappers that are rendered with `un-cloak` in generated HTML.
 *
 * These are the elements whose class lists UnoCSS scans and for which utility
 * CSS is generated. Cloaking is added at HTML generation time to avoid a first
 * paint flash before this runtime module executes.
 */
const UNO_CLOAK_SELECTOR =
  'stencila-styled-block > [slot="content"][un-cloak], stencila-styled-inline > [slot="content"][un-cloak]'

/**
 * Temporary class added when the timeout fallback reveals any still-cloaked
 * elements so that base CSS can animate their opacity rather than letting them
 * appear abruptly.
 */
const UNO_FADEIN_CLASS = 'stencila-unocss-uncloaking'

/**
 * Safety timeout for cases where UnoCSS does not remove `un-cloak` itself.
 *
 * In the normal path, UnoCSS should process and reveal cloaked elements.
 * This fallback prevents content from remaining hidden indefinitely if runtime
 * scanning misses an element or is slower than expected.
 */
const UNO_UNCLOAK_TIMEOUT_MS = 1500

/**
 * Force-reveal any remaining cloaked styled content.
 *
 * This is an escape hatch rather than the primary mechanism. It removes the
 * server-rendered `un-cloak` attribute and adds a class used by CSS to fade the
 * content into view.
 */
function forceUncloak() {
  document.querySelectorAll<HTMLElement>(UNO_CLOAK_SELECTOR).forEach((element) => {
    element.classList.add(UNO_FADEIN_CLASS)
    element.removeAttribute('un-cloak')
  })
}

/**
 * Schedule the fallback reveal after UnoCSS has had time to initialize and scan.
 */
function scheduleForceUncloak() {
  window.setTimeout(forceUncloak, UNO_UNCLOAK_TIMEOUT_MS)
}

/**
 * Re-schedule uncloaking after client-side page swaps.
 *
 * Glide restores cached HTML fragments, which may still include the
 * server-rendered `un-cloak` attributes. Listening for the post-swap
 * event ensures those newly inserted elements are revealed as well.
 */
function hookGlideUncloak() {
  if (glideHooked) {
    return
  }

  window.addEventListener(GlideEvents.AFTER_SWAP, () => {
    scheduleForceUncloak()
  })

  glideHooked = true
}

/**
 * Boot the UnoCSS runtime once for the lifetime of the page.
 */
function initUnoRuntime() {
  if (unoInitialized) {
    return
  }

  unoInitialized = true

  initUnocssRuntime({
    defaults: {
      presets: [
        presetWind4(),
        presetIcons({
          // Bundled icon sets loaded async on first request for icon from that set
          // These will be served from Stencila's embedded server (e.g. when previewing
          // a site when offline) and should be preferred for any UI icons
          collections: {
            lucide: () => import('@iconify-json/lucide/icons.json'),
            bi: () => import('@iconify-json/bi/icons.json'),
          },
          // Fallback to CDN if icon is not from these icon sets
          cdn: 'https://esm.sh/'
        }),
      ],
    },
  })
}

/**
 * Configuration for UnoCSS runtime with Tailwind v4 compatibility
 *
 * This configuration provides runtime CSS generation for styled nodes. UnoCSS
 * will automatically scan the DOM for class names and generate CSS on-the-fly.
 *
 * To reduce flash of unstyled content, styled block and inline content is
 * rendered with `un-cloak` in the generated HTML. UnoCSS is expected to remove
 * that attribute once it has processed those elements. A timeout fallback is
 * also scheduled here so content does not remain hidden if UnoCSS misses it.
 *
 * Users can add a class for any of the https://icon-sets.iconify.design/ icons
 * (e.g. `i-lucide:clock`) to `StyledInline` nodes and these will be dynamically
 * loaded.
 */
export function initUno() {
  hookGlideUncloak()
  initUnoRuntime()
  scheduleForceUncloak()
}
