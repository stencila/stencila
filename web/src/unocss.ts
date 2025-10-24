import { presetIcons } from '@unocss/preset-icons/dist/browser.mjs'
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
        presetIcons({ cdn: 'https://esm.sh/' }),
      ]
    },
  })
}
