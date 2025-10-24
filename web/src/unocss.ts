import initUnocssRuntime from '@unocss/runtime'
import presetWind4 from '@unocss/preset-wind4'

/**
 * Configuration for UnoCSS runtime with Tailwind v4 compatibility
 *
 * This configuration provides runtime CSS generation for styled nodes.
 * UnoCSS will automatically scan the DOM for class names and generate CSS on-the-fly.
 */
export function initUno() {
  initUnocssRuntime({
    defaults: {
      presets: [
        presetWind4(),
      ]
    },
  })
}
