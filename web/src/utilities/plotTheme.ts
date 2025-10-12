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
  // Dimensions for plot sizing and aspect ratio
  dimensions: {
    aspectRatio: number  // Width/height ratio (e.g., 1.5 for 3:2 landscape, 1.777 for 16:9)
    width: string        // CSS length with units (e.g., "8in", "20cm", "800px")
    height: string       // CSS length with units (usually calculated from width / aspectRatio)
    dpi: number         // Dots per inch for PNG rendering (e.g., 100)
    heightMin: string   // Minimum height for web display (e.g., "300px")
    heightMax: string   // Maximum height for web display (e.g., "800px")
  }

  // Color palette for categorical data
  palette: string[]

  // Shape palette for categorical data
  shapes: string[]

  // Line type palette for categorical data
  lineTypes: string[]

  // Ramp endpoints for continuous/discrete scales
  ramp: {
    start: string
    end: string
  }

  // Surfaces and backgrounds
  background: string
  panel: string
  panelBorderColor: string
  panelBorderWidth: number
  grid: string
  zero: string

  // Typography
  textColor: string
  fontFamily: string
  fontSize: number
  titleSize: number
  subtitleSize: number

  // Layout
  padding: {
    top: number
    right: number
    bottom: number
    left: number
  }

  // Axes and grid
  axis: {
    lineColor: string
    lineWidth: number
    titleColor: string
    titleSize: number
    titleWeight: string
    gridColor: string
    gridWidth: number
    gridXWidth: number
    gridYWidth: number
  }

  // Legends
  legend: {
    background: string
    textColor: string
    textSize: number
    borderColor: string
    borderWidth: number
    position: string
  }

  // Tooltips
  tooltip: {
    background: string
    textColor: string
  }

  // Marks
  mark: {
    pointOpacity: number
    pointSize: number
    lineWidth: number
    areaOpacity: number
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
 * Build plot theme from CSS custom properties (with caching)
 *
 * Reads all --plot-* CSS variables and structures them into a PlotTokens object.
 * Returns null if --plot-theme is set to 'none', allowing libraries to use their defaults.
 */
export function buildPlotTheme(rootElement: HTMLElement): PlotTokens | null {
  const getVar = (name: string, fallback: string = '') =>
    getCSSVariable(rootElement, name, fallback)

  // Check if theming is disabled
  const plotTheme = getVar('--plot-theme', 'custom').trim()
  if (plotTheme === 'none') {
    return null
  }

  // Return cached theme if available
  if (cachedTheme) {
    return cachedTheme
  }

  // Build palette array from individual color variables
  const palette: string[] = []
  for (let i = 1; i <= 12; i++) {
    const color = colorToHex(getVar(`--plot-color-${i}`))
    if (color) palette.push(color)
  }

  // Build shapes array from individual shape variables (8 cross-library compatible shapes)
  const shapes: string[] = []
  for (let i = 1; i <= 8; i++) {
    const shape = getVar(`--plot-shape-${i}`)
    if (shape) shapes.push(shape)
  }

  // Build line types array from individual line type variables (6 cross-library compatible types)
  const lineTypes: string[] = []
  for (let i = 1; i <= 6; i++) {
    const lineType = getVar(`--plot-line-type-${i}`)
    if (lineType) lineTypes.push(lineType)
  }

  // Build and cache the theme
  cachedTheme = {
    // Dimensions
    dimensions: {
      aspectRatio: parseNum(getVar('--plot-aspect-ratio'), 1.5, rootElement),
      width: getVar('--plot-width', '8in'),
      height: getVar('--plot-height', '5.33in'),
      dpi: parseNum(getVar('--plot-dpi'), 100, rootElement),
      heightMin: getVar('--plot-height-min', '300px'),
      heightMax: getVar('--plot-height-max', '800px'),
    },

    // Palette
    palette,

    // Shapes
    shapes,

    // Line types
    lineTypes,

    // Ramp
    ramp: {
      start: colorToHex(getVar('--plot-ramp-start')) || '#000000',
      end: colorToHex(getVar('--plot-ramp-end')) || '#ffffff',
    },

    // Surfaces
    background: colorToHex(getVar('--plot-background')) || '#ffffff',
    panel: colorToHex(getVar('--plot-panel')) || '#ffffff',
    panelBorderColor: colorToHex(getVar('--plot-panel-border-color')) || '#e5e5e5',
    panelBorderWidth: (() => {
      const showBorder = getVar('--plot-panel-border', 'true').trim().toLowerCase()
      if (showBorder === 'false' || showBorder === '0') return 0
      // When enabled, use --border-width-default
      return parseNum(getVar('--border-width-default'), 1, rootElement)
    })(),
    grid: colorToHex(getVar('--plot-grid')) || '#e5e5e5',
    zero: colorToHex(getVar('--plot-zero-line-color')) || '#999999',

    // Typography
    textColor: colorToHex(getVar('--plot-text-color')) || '#000000',
    fontFamily: getVar('--plot-font-family') || 'sans-serif',
    fontSize: parseNum(getVar('--plot-font-size'), 12, rootElement),
    titleSize: parseNum(getVar('--plot-title-size'), 14, rootElement),
    subtitleSize: parseNum(getVar('--plot-subtitle-size'), 12, rootElement),

    // Layout
    padding: {
      top: parseNum(getVar('--plot-padding-top'), 8, rootElement),
      right: parseNum(getVar('--plot-padding-right'), 12, rootElement),
      bottom: parseNum(getVar('--plot-padding-bottom'), 24, rootElement),
      left: parseNum(getVar('--plot-padding-left'), 32, rootElement),
    },

    // Axes
    axis: {
      lineColor: colorToHex(getVar('--plot-axis-line-color')) || '#e5e5e5',
      lineWidth: parseNum(getVar('--plot-axis-line-width'), 1, rootElement),
      titleColor: colorToHex(getVar('--plot-axis-title-color')) || '#000000',
      titleSize: parseNum(getVar('--plot-axis-title-size'), 11, rootElement),
      titleWeight: getVar('--plot-axis-title-weight') || 'bold',
      gridColor: colorToHex(getVar('--plot-grid-color')) || '#e5e5e5',
      gridWidth: parseNum(getVar('--plot-grid-width'), 1, rootElement),
      gridXWidth: parseNum(getVar('--plot-grid-x-width'), 1, rootElement),
      gridYWidth: parseNum(getVar('--plot-grid-y-width'), 1, rootElement),
    },

    // Legend
    legend: {
      background: colorToHex(getVar('--plot-legend-background')) || '#ffffff',
      textColor: colorToHex(getVar('--plot-legend-text-color')) || '#000000',
      textSize: parseNum(getVar('--plot-legend-text-size'), 11, rootElement),
      borderColor: colorToHex(getVar('--plot-legend-border-color')) || '#cccccc',
      borderWidth: parseNum(getVar('--plot-legend-border-width'), 1, rootElement),
      position: getVar('--plot-legend-position') || 'auto',
    },

    // Tooltip
    tooltip: {
      background: colorToHex(getVar('--plot-tooltip-background')) || '#f5f5f5',
      textColor: colorToHex(getVar('--plot-tooltip-text-color')) || '#000000',
    },

    // Marks
    mark: {
      pointOpacity: parseNum(getVar('--plot-point-opacity'), 0, rootElement),
      pointSize: parseNum(getVar('--plot-point-size'), 6, rootElement),
      lineWidth: parseNum(getVar('--plot-line-width'), 2, rootElement),
      areaOpacity: parseNum(getVar('--plot-area-opacity'), 0.25, rootElement),
    },
  }

  return cachedTheme
}
