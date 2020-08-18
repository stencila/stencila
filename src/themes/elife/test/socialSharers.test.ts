import * as socialSharers from '../lib/socialSharers'

const body = document.body
const resetDom = (): void => {
  body.innerHTML = ''
}

describe('social sharing links', () => {
  const title =
    'Replication Study: Transcriptional amplification in tumor cells with elevated c-Myc'
  const id = '30274'
  let sharers: Element | null

  beforeEach(() => {
    sharers = socialSharers.build(title, id, body)
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

  describe('Facebook link', () => {
    let link: HTMLAnchorElement | null | undefined

    beforeEach(() => {
      link = sharers?.querySelector('a[href^="https://facebook.com/"]')
    })

    it('passes the DOI to Facebook', () => {
      const expectedUrl =
        'https://facebook.com/sharer/sharer.php?u=https%3A%2F%2Felifesciences.org%2Farticles%2F30274%2Fexecutable'
      expect(
        link?.getAttribute('href') ?? 'no appropriate anchor found with href'
      ).toBe(expectedUrl)
    })

    it('has the aria-label "Share on Facebook"', () => {
      expect(
        link?.getAttribute('aria-label') ??
          'no appropriate anchor found with aria-label'
      ).toBe('Share on Facebook')
    })
  })
  describe('Twitter link', () => {
    let link: HTMLAnchorElement | null | undefined

    beforeEach(() => {
      link = sharers?.querySelector('a[href^="https://twitter.com/"]')
    })

    it('passes the title and DOI to Twitter', () => {
      const expectedUrl =
        'https://twitter.com/intent/tweet/?text=Replication%20Study%3A%20Transcriptional%20amplification%20in%20tumor%20cells%20with%20elevated%20c-Myc&url=https%3A%2F%2Felifesciences.org%2Farticles%2F30274%2Fexecutable'
      expect(link?.getAttribute('href') ?? 'no appropriate anchor found').toBe(
        expectedUrl
      )
    })
    it('has the aria-label "Tweet a link to this page"', () => {
      expect(
        link?.getAttribute('aria-label') ??
          'no appropriate anchor found with aria-label'
      ).toBe('Tweet a link to this page')
    })
  })

  describe('Reddit link', () => {
    let link: HTMLAnchorElement | null | undefined

    beforeEach(() => {
      link = sharers?.querySelector('a[href^="https://reddit.com"]')
    })

    it('passes the title and DOI to Reddit', () => {
      const expectedUrl =
        'https://reddit.com/submit/?title=Replication%20Study%3A%20Transcriptional%20amplification%20in%20tumor%20cells%20with%20elevated%20c-Myc&url=https%3A%2F%2Felifesciences.org%2Farticles%2F30274%2Fexecutable'
      expect(link?.getAttribute('href') ?? 'no appropriate anchor found').toBe(
        expectedUrl
      )
    })

    it('has the aria-label "Share this page on Reddit"', () => {
      expect(
        link?.getAttribute('aria-label') ??
          'no appropriate anchor found with aria-label'
      ).toBe('Share this page on Reddit')
    })
  })

  describe('email link', () => {
    let link: HTMLAnchorElement | null | undefined

    beforeEach(() => {
      link = sharers?.querySelector('a[href^="mailto:"]')
    })

    it('is a mailto, with the title as the subject and the DOI as the body', () => {
      const expectedUrl =
        'mailto:?subject=Replication%20Study%3A%20Transcriptional%20amplification%20in%20tumor%20cells%20with%20elevated%20c-Myc&body=https%3A%2F%2Felifesciences.org%2Farticles%2F30274%2Fexecutable'
      expect(link?.getAttribute('href') ?? 'no appropriate anchor found').toBe(
        expectedUrl
      )
    })

    it('has the aria-label "Email a link to this page (opens up email program, if configured on this system)"', () => {
      expect(
        link?.getAttribute('aria-label') ??
          'no appropriate anchor found with aria-label'
      ).toBe(
        'Email a link to this page (opens up email program, if configured on this system)'
      )
    })
  })
})
