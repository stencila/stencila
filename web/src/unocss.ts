import { presetIcons } from '@unocss/preset-icons/browser'
import { presetWind4 } from '@unocss/preset-wind4'
import initUnocssRuntime from '@unocss/runtime'

/**
 * Configuration for UnoCSS runtime with Tailwind v4 compatibility
 *
 * This configuration provides runtime CSS generation for styled nodes. UnoCSS
 * will automatically scan the DOM for class names and generate CSS on-the-fly.
 *
 * Users can add a class for any of the https://icon-sets.iconify.design/ icons
 * (e.g. `i-lucide:clock`) to `StyledInline` nodes and these will be dynamically
 * loaded.
 */
export function initUno() {
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
