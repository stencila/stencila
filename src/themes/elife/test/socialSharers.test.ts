import * as socialSharers from '../lib/socialSharers'

const body = document.body
const resetDom = (): void => {
  body.innerHTML = ''
}

describe('social sharing links', () => {
  const title =
    'Replication Study: Transcriptional amplification in tumor cells with elevated c-Myc'
  const doi = '10.7554/eLife.30274'
  let sharers: Element | null

  beforeEach(() => {
    sharers = socialSharers.build(title, doi, body)
    if (sharers === null) {
      throw new Error('No social sharing links found')
    }
  })

  afterEach(resetDom)

  it('has 4 links', () => {
    expect(sharers?.querySelectorAll('a').length ?? 0).toBe(4)
  })

  it('are all SVG images', () => {
    Array.from(sharers?.children ?? []).forEach((element: Element) => {
      expect(element.firstElementChild instanceof SVGElement).toBe(true)
      expect(element.childElementCount).toBe(1)
    })
  })

  it('has a link that passes the DOI to Facebook', () => {
    const sharer = sharers?.querySelector('a[href^="https://facebook.com/"]')
    const expectedUrl =
      'https://facebook.com/sharer/sharer.php?u=https://doi.org/10.7554/eLife.30274'
    expect(
      sharer?.getAttribute('href') ?? 'no appropriate anchor found'
    ).toEqual(expectedUrl)
  })

  it('has a link that passes the title and DOI to Twitter', () => {
    const sharer = sharers?.querySelector('a[href^="https://twitter.com/"]')
    const expectedUrl =
      'https://twitter.com/intent/tweet/?text=Replication%20Study%3A%20Transcriptional%20amplification%20in%20tumor%20cells%20with%20elevated%20c-Myc&amp;url=https://doi.org/10.7554/eLife.30274'
    expect(
      sharer?.getAttribute('href') ?? 'no appropriate anchor found'
    ).toEqual(expectedUrl)
  })

  it('has a link that passes the title and DOI to Reddit', () => {
    const sharer = sharers?.querySelector('a[href^="https://reddit.com"]')
    const expectedUrl =
      'https://reddit.com/submit/?title=Replication%20Study%3A%20Transcriptional%20amplification%20in%20tumor%20cells%20with%20elevated%20c-Myc&amp;url=https://doi.org/10.7554/eLife.30274'
    expect(
      sharer?.getAttribute('href') ?? 'no appropriate anchor found'
    ).toEqual(expectedUrl)
  })

  it('has a link to open email with the title as the subject and the DOI as the body', () => {
    const sharer = sharers?.querySelector('a[href^="mailto:"]')
    const expectedUrl =
      'mailto:?subject=Replication%20Study%3A%20Transcriptional%20amplification%20in%20tumor%20cells%20with%20elevated%20c-Myc&amp;body=https://doi.org/10.7554/eLife.30274'
    expect(
      sharer?.getAttribute('href') ?? 'no appropriate anchor found'
    ).toEqual(expectedUrl)
  })
})
