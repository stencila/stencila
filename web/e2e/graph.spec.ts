import { readFileSync } from 'node:fs'

import { argosScreenshot } from '@argos-ci/playwright'
import { type Locator, type Page, expect, test } from '@playwright/test'

const fixture = JSON.parse(
  readFileSync(
    new URL('../../examples/web/graph/lineage.json', import.meta.url),
    'utf8'
  )
)

async function graphReady(page: Page) {
  await page.addInitScript(() =>
    localStorage.setItem('stencila-color-scheme-preference', 'light')
  )
  await page.goto('/graph/lineage.json?~view=graph')
  const graph = page.locator('stencila-graph-view')
  await graph.evaluate(
    (
      element: Element & { error?: string; graph?: unknown },
      deterministicGraph
    ) => {
      element.error = undefined
      element.graph = deterministicGraph
    },
    fixture
  )
  await expect(graph.locator('.canvas')).toBeVisible()
  await expect
    .poll(() =>
      graph.evaluate((element: Element & { cy?: { nodes(): unknown[] } }) =>
        element.cy?.nodes().length ?? 0
      )
    )
    .toBe(4)
  const fonts = await graph.evaluate(
    (
      element: Element & {
        cy?: {
          edges(): { first(): { style(name: string): string } }
          nodes(): { first(): { style(name: string): string } }
        }
      }
    ) => ({
      interface: getComputedStyle(element).fontFamily,
      node: element.cy?.nodes().first().style('font-family'),
      edge: element.cy?.edges().first().style('font-family'),
    })
  )
  expect(fonts.node).toBe(fonts.interface)
  expect(fonts.edge).toBe(fonts.interface)
  return graph
}

async function graphSurfaceLightness(graph: Locator) {
  return graph.evaluate((element) => {
    const lightness = (selector: string, property = 'backgroundColor') => {
      const target = element.shadowRoot?.querySelector(selector)
      if (!target) {
        return undefined
      }

      const color = getComputedStyle(target)[
        property as 'backgroundColor' | 'color'
      ]
      const channels = color.match(/[\d.]+/g)?.slice(0, 3).map(Number)
      if (!channels || channels.length < 3) {
        return undefined
      }

      const scale = Math.max(...channels) <= 1 ? 1 : 255
      return (
        (0.2126 * channels[0] +
          0.7152 * channels[1] +
          0.0722 * channels[2]) /
        scale
      )
    }

    return {
      colorScheme: getComputedStyle(element).colorScheme,
      inspector: lightness('.inspector'),
      inspectorText: lightness('.inspector', 'color'),
      legend: lightness('.legend'),
      record: lightness('.evidence-disclosure'),
    }
  })
}

async function emitOnKind(
  page: Page,
  kind: string,
  event: 'mouseover' | 'tap'
) {
  await page.locator('stencila-graph-view').evaluate(
    (
      element: Element & {
        cy?: {
          edges(): {
            filter(selector: string): {
              first(): {
                renderedMidpoint(): { x: number; y: number }
                emit(event: object): void
              }
            }
          }
        }
      },
      args
    ) => {
      const edge = element.cy
        ?.edges()
        .filter(`[kind = "${args.kind}"]`)
        .first()
      edge?.emit({
        type: args.event,
        renderedPosition: edge.renderedMidpoint(),
      })
    },
    { kind, event }
  )
}

async function emitOnNode(page: Page, id: string) {
  await page.locator('stencila-graph-view').evaluate(
    (
      element: Element & {
        cy?: { getElementById(id: string): { emit(event: string): void } }
      },
      nodeId
    ) => element.cy?.getElementById(nodeId).emit('tap'),
    id
  )
}

async function emitOnCanvas(page: Page) {
  await page.locator('stencila-graph-view').evaluate(
    (
      element: Element & {
        cy?: { emit(event: { type: string; target: unknown }): void }
      }
    ) => element.cy?.emit({ type: 'tap', target: element.cy })
  )
}

test('previews and pins complete edge evidence', async ({ page }) => {
  test.setTimeout(180_000)
  const graph = await graphReady(page)

  const renderedEvidence = await graph.evaluate(
    (
      element: Element & {
        cy?: {
          edges(): {
            filter(selector: string): {
              first(): {
                data(name: string): unknown
                style(name: string): string
              }
            }
          }
        }
      }
    ) => {
      const edge = element.cy?.edges().filter('[kind = "ReadBy"]').first()
      return {
        label: edge?.data('displayLabel'),
        midpointMarker: edge?.style('mid-target-arrow-shape'),
      }
    }
  )
  expect(renderedEvidence).toEqual({
    label: 'Read By · ● ◆',
    midpointMarker: 'none',
  })

  await emitOnKind(page, 'ReadBy', 'mouseover')
  await expect(graph.locator('[role=tooltip]')).toContainText(
    'StaticAnalysis (high), Attested (certain)'
  )
  await expect(graph.locator('[role=tooltip]')).toContainText(
    '2 evidence items · 1 activity · 2 contributing relationships'
  )

  await emitOnKind(page, 'ReadBy', 'tap')
  const inspector = graph.locator('[role=complementary]')
  await expect(inspector).toBeVisible()
  await expect(inspector).toContainText('data.csv → analysis.py')
  await expect(inspector.locator('.evidence-group .section-title')).toHaveText(
    'Evidence (2)'
  )
  await expect(inspector).toContainText('Contributing relationships (2)')
  await expect(inspector.locator('.relationship-row')).toHaveCount(2)
  await expect(inspector.locator('.relationship-row .badge.gray')).toHaveText([
    '#1',
    '#2',
  ])
  await expect(inspector).toContainText('analysis.py:4:8')
  await expect(inspector).toContainText('Timestamp')
  await expect(inspector).toContainText('Millisecond')
  await expect(inspector).not.toContainText('[object Object]')
  await expect(inspector).toContainText('Associated activities (1)')
  await expect(
    inspector.getByText('Export document', { exact: true })
  ).toBeVisible()
  await expect(inspector.locator('.metadata-null')).toContainText('—')
  await expect(inspector.locator('.metadata-item-heading')).toHaveText([
    'uv.lock',
    'pyproject.toml',
    'stencila',
  ])
  await expect(inspector.locator('.metadata-link')).toHaveAttribute(
    'href',
    'https://github.com/stencila/stencila'
  )
  await expect(inspector.locator('.metadata-identifier')).toHaveCount(2)
  await expect(inspector).not.toContainText('{"architecture"')
  const evidenceDisclosures = inspector.locator('.evidence-disclosure')
  const activityDisclosures = inspector.locator('.activity-disclosure')
  await expect(evidenceDisclosures).toHaveCount(2)
  await expect(
    evidenceDisclosures.first().locator('.metadata-type')
  ).toHaveText(['Array validator', 'String validator'])
  await expect(
    evidenceDisclosures.first().getByText('Type', { exact: true })
  ).toHaveCount(0)
  const nestedLayout = await evidenceDisclosures
    .first()
    .locator('.metadata-grid')
    .evaluateAll((grids) =>
      grids.map((grid) => ({
        columns: getComputedStyle(grid).gridTemplateColumns.split(/\s+/).length,
        depth: grid.classList.contains('metadata-depth-2') ? 2 : 1,
        padding: getComputedStyle(grid).paddingLeft,
      }))
    )
  expect(nestedLayout.every(({ columns }) => columns === 1)).toBe(true)
  expect(
    nestedLayout
      .filter(({ depth }) => depth === 2)
      .every(({ padding }) => padding === '0px')
  ).toBe(true)
  await expect(evidenceDisclosures.first()).toHaveAttribute('open', '')
  await expect(evidenceDisclosures.nth(1)).toHaveAttribute('open', '')
  await expect(activityDisclosures).toHaveAttribute('open', '')
  await evidenceDisclosures.first().locator('summary').click()
  await expect(evidenceDisclosures.first()).not.toHaveAttribute('open', '')
  await expect(evidenceDisclosures.nth(1)).toHaveAttribute('open', '')
  await graph.evaluate(
    (element: Element & { requestUpdate(): void }) => element.requestUpdate()
  )
  await expect(evidenceDisclosures.first()).not.toHaveAttribute('open', '')
  await expect(inspector).not.toContainText('Machine-readable details')
  await expect(inspector.locator('.edge-record, .evidence-record')).toHaveCount(
    0
  )

  const legend = graph.locator('.legend')
  await expect(legend).not.toContainText('Color shows role')
  await expect(legend.locator('.legend-group')).toHaveCount(3)
  await expect(legend.locator('.legend-group-title')).toHaveText([
    'Relationships',
    'Nodes',
    'Evidence & selection',
  ])
  await expect(legend.locator('.node-shape')).toHaveCount(7)
  await expect(legend).toContainText('Read or include')
  await expect(legend).toContainText('Generated output')
  await expect(legend).toContainText('●●●')

  await argosScreenshot(page, 'graph-lineage-light')
  await inspector.getByRole('button', { name: 'Close graph inspector' }).click()
  await expect(inspector).toBeHidden()
  await graph
    .getByRole('button', { name: 'Switch to dark mode' })
    .click()
  await expect(page.locator('html')).toHaveAttribute(
    'data-color-scheme',
    'dark'
  )
  await emitOnKind(page, 'ReadBy', 'tap')
  await expect(inspector).toBeVisible()
  await expect
    .poll(async () => (await graphSurfaceLightness(graph)).inspector)
    .toBeLessThan(0.25)
  const darkSurfaces = await graphSurfaceLightness(graph)
  expect(darkSurfaces).toMatchObject({ colorScheme: 'dark' })
  expect(darkSurfaces.inspectorText).toBeGreaterThan(0.6)
  expect(darkSurfaces.legend).toBeLessThan(0.25)
  expect(darkSurfaces.record).toBeLessThan(0.3)
  await argosScreenshot(page, 'graph-lineage-dark')

  await emitOnKind(page, 'Generated', 'tap')
  await expect(inspector.locator('.panel-title')).toHaveText(
    'analysis.py → summary.csv'
  )
  await expect(inspector.locator('.inspector-header .badge')).toHaveText(
    'Generated'
  )
  await expect(inspector).toContainText('1 evidence item · 1 activity')
  await expect(inspector).not.toContainText('Contributing relationships')
  await expect(inspector.locator('.evidence-disclosure')).toHaveCount(1)
  await expect(inspector.locator('.evidence-disclosure')).toHaveAttribute(
    'open',
    ''
  )
  await expect(inspector.locator('.activity-disclosure')).toHaveAttribute(
    'open',
    ''
  )

  await emitOnNode(page, 'output:summary.csv')
  await expect(inspector).toContainText('Properties')
  await expect(inspector).toContainText('output:summary.csv')
  await expect(inspector).toContainText('File')
  await expect(inspector.locator('.properties-disclosure')).toHaveAttribute(
    'open',
    ''
  )
  await expect(inspector).not.toContainText('Schema type')
  const propertiesSummary = inspector.locator(
    '.properties-disclosure > summary'
  )
  await propertiesSummary.focus()
  await expect(propertiesSummary).toBeFocused()

  await emitOnCanvas(page)
  await expect(inspector).toBeVisible()
  await expect(inspector).toContainText('output:summary.csv')

  await graph.evaluate(
    (element: Element & { includeStructureEdges?: boolean }) => {
      element.includeStructureEdges = true
    }
  )
  await expect
    .poll(() =>
      graph.evaluate(
        (
          element: Element & {
            cy?: { edges(selector: string): { length: number } }
          }
        ) => element.cy?.edges('[kind = "PartOf"]').length ?? 0
      )
    )
    .toBe(1)
  await emitOnKind(page, 'PartOf', 'tap')
  await expect(inspector.locator('.evidence-group .section-title')).toHaveText(
    'Evidence (0)'
  )
  await expect(inspector).toContainText('No recorded evidence')
  await expect(inspector).toContainText('Associated activities (0)')

  await inspector.getByRole('button', { name: 'Close graph inspector' }).click()
  await expect(inspector).toBeHidden()
})

test('traces and clears upstream and downstream dependencies', async ({
  page,
}) => {
  test.setTimeout(120_000)
  const graph = await graphReady(page)
  const result = await graph.evaluate(
    (
      element: Element & {
        cy?: {
          getElementById(id: string): { emit(event: string): void }
          elements(selector: string): { length: number }
        }
      }
    ) => {
      element.cy?.getElementById('output:summary.csv').emit('tap')
      return {
        upstreamNodes: element.cy?.elements('.lineage-upstream-node').length,
        downstreamNodes: element.cy?.elements('.lineage-downstream-node')
          .length,
        upstreamEdges: element.cy?.elements('.lineage-upstream-edge').length,
        downstreamEdges: element.cy?.elements('.lineage-downstream-edge')
          .length,
        selected: element.cy?.elements('.lineage-selected').length,
        inspectedNodes: element.cy?.elements('.inspector-selected-node').length,
        tracedAndInspectedNode: element.cy?.elements(
          '.lineage-selected.inspector-selected-node'
        ).length,
      }
    }
  )

  expect(result).toEqual({
    upstreamNodes: 2,
    downstreamNodes: 1,
    upstreamEdges: 2,
    downstreamEdges: 1,
    selected: 1,
    inspectedNodes: 1,
    tracedAndInspectedNode: 1,
  })

  await emitOnKind(page, 'ReadBy', 'tap')
  const combined = await graph.evaluate(
    (
      element: Element & {
        cy?: { elements(selector: string): { length: number } }
      }
    ) => ({
      inspectedEdges: element.cy?.elements('.inspector-selected-edge').length,
      tracedAndInspected: element.cy?.elements(
        '.lineage-upstream-edge.inspector-selected-edge'
      ).length,
      tracedEdges: element.cy?.elements('.lineage-upstream-edge').length,
    })
  )
  expect(combined).toEqual({
    inspectedEdges: 1,
    tracedAndInspected: 1,
    tracedEdges: 2,
  })

  await graph
    .locator('[role=complementary]')
    .getByRole('button', { name: 'Close graph inspector' })
    .click()
  await expect
    .poll(() =>
      graph.evaluate(
        (
          element: Element & {
            cy?: { elements(selector: string): { length: number } }
          }
        ) => ({
          inspected: element.cy?.elements('.inspector-selected-edge').length,
          traced: element.cy?.elements('.lineage-upstream-edge').length,
        })
      )
    )
    .toEqual({ inspected: 0, traced: 2 })

  await graph.evaluate(
    (
      element: Element & {
        cy?: { getElementById(id: string): { emit(event: string): void } }
      }
    ) => element.cy?.getElementById('output:summary.csv').emit('tap')
  )
  await expect
    .poll(() =>
      graph.evaluate(
        (
          element: Element & {
            cy?: { elements(selector: string): { length: number } }
          }
        ) =>
          element.cy?.elements(
            '.lineage-upstream-node, .lineage-downstream-node, .lineage-selected'
          ).length ?? 0
      )
    )
    .toBe(0)
})

test('adjusts layout spacing while preserving graph selections', async ({
  page,
}) => {
  test.setTimeout(120_000)
  const graph = await graphReady(page)
  await emitOnNode(page, 'output:summary.csv')
  await graph.getByRole('button', { name: 'Graph settings' }).click()

  const spacing = graph.getByRole('slider', { name: 'Graph spacing' })
  await expect(spacing).toHaveValue('1')
  await expect(graph.locator('output[for=graph-spacing]')).toHaveText('cozy')

  await spacing.fill('4')
  await expect(graph.locator('output[for=graph-spacing]')).toHaveText('spacious')
  await expect
    .poll(() =>
      graph.evaluate(
        (
          element: Element & {
            cy?: {
              elements(selector: string): { length: number }
              options(): { layout: { spacingFactor?: number } }
            }
            layoutSpacing?: string
          }
        ) => ({
          inspected: element.cy?.elements('.inspector-selected-node').length,
          selected: element.cy?.elements('.lineage-selected').length,
          spacing: element.layoutSpacing,
          spacingFactor: element.cy?.options().layout.spacingFactor,
        })
      )
    )
    .toEqual({
      inspected: 1,
      selected: 1,
      spacing: 'spacious',
      spacingFactor: 1.65,
    })
})

test('uses a bottom sheet and collapsed legend on mobile', async ({ page }) => {
  test.setTimeout(120_000)
  await page.setViewportSize({ width: 390, height: 844 })
  const graph = await graphReady(page)
  const legend = graph.locator('.legend')
  const legendToggle = legend.locator('summary').first()
  await expect(legend).not.toHaveAttribute('open', '')
  const collapsedToggle = await legendToggle.boundingBox()
  await legendToggle.click()
  await expect(legend).toHaveAttribute('open', '')
  const expandedToggle = await legendToggle.boundingBox()
  expect(Math.round(expandedToggle?.y ?? -1)).toBe(
    Math.round(collapsedToggle?.y ?? -2)
  )
  await expect(legend.locator('.legend-group')).toHaveCount(3)
  await expect
    .poll(async () => Math.round((await legend.boundingBox())?.width ?? 0))
    .toBeLessThanOrEqual(390)
  await argosScreenshot(page, 'graph-lineage-mobile-legend')
  await legendToggle.click()

  await emitOnKind(page, 'Generated', 'tap')
  const inspector = graph.locator('[role=complementary]')
  await expect(inspector).toBeVisible()
  const viewportHeight = await page.evaluate(() => window.innerHeight)
  await expect
    .poll(async () => {
      const box = await inspector.boundingBox()
      return box && {
        width: Math.round(box.width),
        bottom: Math.round(box.y + box.height),
      }
    })
    .toEqual({ width: 390, bottom: viewportHeight })

  await inspector.getByRole('button', { name: 'Close graph inspector' }).click()
  await expect(inspector).toBeHidden()
})
