import { colorToHex } from './colorUtils'
import { getCSSVariable } from './cssVariables'

/**
 * Module-level theme cache
 */
let cachedTheme: PlotTokens | null = null

/**
 * Clear cache on theme changes
 */
if (typeof window !== 'undefined') {
  window.addEventListener('stencila-color-scheme-changed', () => {
    cachedTheme = null
  })
  window.addEventListener('stencila-theme-changed', () => {
    cachedTheme = null
  })
}

/**
 * Plot theme tokens built from CSS custom properties
 *
 * Structured to match the plotting design tokens in plots.css
 */
export interface PlotTokens {
  // Color palette for categorical data
  palette: string[]

  // Ramp endpoints for continuous/discrete scales
  ramp: {
    start: string
    end: string
    steps: number
  }

  // Semantic colors
  positive: string
  negative: string
  warning: string

  // Surfaces and backgrounds
  background: string
  panel: string
  muted: string
  grid: string
  contrastGrid: string
  zero: string

  // Typography
  textColor: string
  fontFamily: string
  fontMono: string
  fontSize: number
  titleSize: number
  subtitleSize: number
  legendSize: number
  axisTitleSize: number

  // Layout
  padding: {
    top: number
    right: number
    bottom: number
    left: number
  }
  gap: {
    x: number
    y: number
  }
  radius: number
  strokeWidth: number

  // Axes and grid
  axis: {
    lineColor: string
    lineWidth: number
    tickColor: string
    tickWidth: number
    tickSize: number
    titleColor: string
    gridColor: string
    gridWidth: number
    gridDash: number
  }

  // Legends
  legend: {
    background: string
    textColor: string
    borderColor: string
    borderWidth: number
    markerSize: number
    gap: number
    position: string
  }

  // Tooltips
  tooltip: {
    background: string
    textColor: string
    borderColor: string
    borderWidth: number
    radius: number
    shadow: string
    padX: number
    padY: number
  }

  // Marks
  mark: {
    opacity: number
    lineJoin: string
    lineCap: string
    lineWidth: number
    lineDash: number
    areaOpacity: number
    pointSize: number
    pointBorderWidth: number
    barGap: number
    barCategoryGap: number
    barRadius: number
    barBorderWidth: number
    candleUp: string
    candleDown: string
    candleWick: string
    heatMin: string
    heatMax: string
  }

  // Interaction
  interaction: {
    crosshairColor: string
    crosshairWidth: number
    crosshairDash: string
    hoverOpacity: number
    focusOutlineColor: string
    focusOutlineWidth: number
    selectionBg: string
    selectionStroke: string
  }

  // Motion
  motion: {
    duration: number
    ease: string
  }
}

/**
 * Parse CSS value to number (converting rem/em to pixels, s to ms)
 */
function parseNum(
  value: string,
  fallback: number = 0,
  rootElement?: HTMLElement
): number {
  if (!value) return fallback

  const parsed = parseFloat(value)
  if (isNaN(parsed)) return fallback

  // If value has rem unit, convert to pixels
  if (value.includes('rem')) {
    const fontSize = rootElement
      ? parseFloat(getComputedStyle(rootElement).fontSize)
      : 16
    return parsed * fontSize
  }

  // If value has em unit, convert to pixels
  if (value.includes('em')) {
    const fontSize = rootElement
      ? parseFloat(getComputedStyle(rootElement).fontSize)
      : 16
    return parsed * fontSize
  }

  // If value has s unit (seconds), convert to milliseconds
  if (value.match(/\ds$/)) {
    return parsed * 1000
  }

  // For px, ms, or unitless values, return as-is
  return parsed
}

/**
 * Parse CSS dash pattern (e.g., "4 2" or "0") to number or array
 */
function parseDash(value: string): number {
  const parsed = parseFloat(value.split(' ')[0])
  return isNaN(parsed) ? 0 : parsed
}

/**
 * Build plot theme from CSS custom properties (with caching)
 *
 * Reads all --plot-* CSS variables and structures them into a PlotTokens object
 */
export function buildPlotTheme(rootElement: HTMLElement): PlotTokens {
  // Return cached theme if available
  if (cachedTheme) {
    return cachedTheme
  }

  const getVar = (name: string, fallback: string = '') =>
    getCSSVariable(rootElement, name, fallback)

  // Build palette array from individual color variables
  const palette: string[] = []
  for (let i = 1; i <= 12; i++) {
    const color = colorToHex(getVar(`--plot-color-${i}`))
    if (color) palette.push(color)
  }

  // Build and cache the theme
  cachedTheme = {
    // Palette
    palette,

    // Ramp
    ramp: {
      start: colorToHex(getVar('--plot-ramp-start')) || '#000000',
      end: colorToHex(getVar('--plot-ramp-end')) || '#ffffff',
      steps: parseNum(getVar('--plot-ramp-steps'), 7, rootElement),
    },

    // Semantic colors
    positive: colorToHex(getVar('--plot-positive')) || '#22c55e',
    negative: colorToHex(getVar('--plot-negative')) || '#ef4444',
    warning: colorToHex(getVar('--plot-warning')) || '#eab308',

    // Surfaces
    background: colorToHex(getVar('--plot-background')) || '#ffffff',
    panel: colorToHex(getVar('--plot-panel')) || '#ffffff',
    muted: colorToHex(getVar('--plot-muted')) || '#666666',
    grid: colorToHex(getVar('--plot-grid')) || '#e5e5e5',
    contrastGrid: colorToHex(getVar('--plot-contrast-grid')) || '#cccccc',
    zero: colorToHex(getVar('--plot-zero-line-color')) || '#999999',

    // Typography
    textColor: colorToHex(getVar('--plot-text-color')) || '#000000',
    fontFamily: getVar('--plot-font-family') || 'sans-serif',
    fontMono: getVar('--plot-font-mono') || 'monospace',
    fontSize: parseNum(getVar('--plot-font-size'), 12, rootElement),
    titleSize: parseNum(getVar('--plot-title-size'), 14, rootElement),
    subtitleSize: parseNum(getVar('--plot-subtitle-size'), 12, rootElement),
    legendSize: parseNum(getVar('--plot-legend-size'), 11, rootElement),
    axisTitleSize: parseNum(getVar('--plot-axis-title-size'), 11, rootElement),

    // Layout
    padding: {
      top: parseNum(getVar('--plot-padding-top'), 8, rootElement),
      right: parseNum(getVar('--plot-padding-right'), 12, rootElement),
      bottom: parseNum(getVar('--plot-padding-bottom'), 24, rootElement),
      left: parseNum(getVar('--plot-padding-left'), 32, rootElement),
    },
    gap: {
      x: parseNum(getVar('--plot-gap-x'), 16, rootElement),
      y: parseNum(getVar('--plot-gap-y'), 16, rootElement),
    },
    radius: parseNum(getVar('--plot-radius'), 4, rootElement),
    strokeWidth: parseNum(getVar('--plot-stroke-width'), 2, rootElement),

    // Axes
    axis: {
      lineColor: colorToHex(getVar('--plot-axis-line-color')) || '#e5e5e5',
      lineWidth: parseNum(getVar('--plot-axis-line-width'), 1, rootElement),
      tickColor: colorToHex(getVar('--plot-tick-color')) || '#666666',
      tickWidth: parseNum(getVar('--plot-tick-width'), 1, rootElement),
      tickSize: parseNum(getVar('--plot-tick-size'), 4, rootElement),
      titleColor: colorToHex(getVar('--plot-axis-title-color')) || '#000000',
      gridColor: colorToHex(getVar('--plot-grid-color')) || '#e5e5e5',
      gridWidth: parseNum(getVar('--plot-grid-width'), 1, rootElement),
      gridDash: parseDash(getVar('--plot-grid-dash', '0')),
    },

    // Legend
    legend: {
      background: colorToHex(getVar('--plot-legend-background')) || '#ffffff',
      textColor: colorToHex(getVar('--plot-legend-text-color')) || '#000000',
      borderColor: colorToHex(getVar('--plot-legend-border-color')) || '#cccccc',
      borderWidth: parseNum(getVar('--plot-legend-border-width'), 1, rootElement),
      markerSize: parseNum(getVar('--plot-legend-marker-size'), 8, rootElement),
      gap: parseNum(getVar('--plot-legend-gap'), 8, rootElement),
      position: getVar('--plot-legend-position') || 'auto',
    },

    // Tooltip
    tooltip: {
      background: colorToHex(getVar('--plot-tooltip-background')) || '#f5f5f5',
      textColor: colorToHex(getVar('--plot-tooltip-text-color')) || '#000000',
      borderColor: colorToHex(getVar('--plot-tooltip-border-color')) || 'transparent',
      borderWidth: parseNum(getVar('--plot-tooltip-border-width'), 0, rootElement),
      radius: parseNum(getVar('--plot-tooltip-radius'), 4, rootElement),
      shadow: getVar('--plot-tooltip-shadow') || '0 2px 8px rgba(0,0,0,0.15)',
      padX: parseNum(getVar('--plot-tooltip-padding-x'), 8, rootElement),
      padY: parseNum(getVar('--plot-tooltip-padding-y'), 6, rootElement),
    },

    // Marks
    mark: {
      opacity: parseNum(getVar('--plot-mark-opacity'), 1, rootElement),
      lineJoin: getVar('--plot-line-join') || 'round',
      lineCap: getVar('--plot-line-cap') || 'round',
      lineWidth: parseNum(getVar('--plot-line-width'), 2, rootElement),
      lineDash: parseDash(getVar('--plot-line-dash', '0')),
      areaOpacity: parseNum(getVar('--plot-area-opacity'), 0.25, rootElement),
      pointSize: parseNum(getVar('--plot-point-size'), 6, rootElement),
      pointBorderWidth: parseNum(getVar('--plot-point-border-width'), 1, rootElement),
      barGap: parseNum(getVar('--plot-bar-gap'), 0.2, rootElement),
      barCategoryGap: parseNum(getVar('--plot-bar-category-gap'), 0.3, rootElement),
      barRadius: parseNum(getVar('--plot-bar-radius'), 2, rootElement),
      barBorderWidth: parseNum(getVar('--plot-bar-border-width'), 0, rootElement),
      candleUp: colorToHex(getVar('--plot-candle-up')) || '#22c55e',
      candleDown: colorToHex(getVar('--plot-candle-down')) || '#ef4444',
      candleWick: colorToHex(getVar('--plot-candle-wick')) || '#666666',
      heatMin: colorToHex(getVar('--plot-heatmap-min')) || '#000000',
      heatMax: colorToHex(getVar('--plot-heatmap-max')) || '#ffffff',
    },

    // Interaction
    interaction: {
      crosshairColor:
        colorToHex(getVar('--plot-crosshair-color')) || 'rgba(0,0,0,0.4)',
      crosshairWidth: parseNum(getVar('--plot-crosshair-width'), 1, rootElement),
      crosshairDash: getVar('--plot-crosshair-dash') || '4 2',
      hoverOpacity: parseNum(getVar('--plot-hover-opacity'), 0.7, rootElement),
      focusOutlineColor: colorToHex(getVar('--plot-focus-outline-color')) || '#3b82f6',
      focusOutlineWidth: parseNum(getVar('--plot-focus-outline-width'), 2, rootElement),
      selectionBg:
        colorToHex(getVar('--plot-selection-bg')) || 'rgba(59,130,246,0.15)',
      selectionStroke: colorToHex(getVar('--plot-selection-stroke')) || '#3b82f6',
    },

    // Motion
    motion: {
      duration: parseNum(getVar('--plot-anim-duration'), 250, rootElement),
      ease: getVar('--plot-anim-ease') || 'cubic-bezier(.2, .8, .2, 1)',
    },
  }

  return cachedTheme
}
