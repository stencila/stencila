import { first, ready, select } from '../../util'
import * as downloads from './downloads'
import * as references from './references'
import DateTimeFormat = Intl.DateTimeFormat

const dateFormatter = new DateTimeFormat('en-US', {
  month: 'short',
  day: 'numeric',
  year: 'numeric',
})

const formatDate = (dateEl: Element | null): void => {
  if (dateEl instanceof Element) {
    const date = new Date(dateEl.innerHTML)
    dateEl.innerHTML = dateFormatter.format(date)
  }
}

const getArticleId = (): string => {
  const selector =
    ':--identifier meta[content="https://registry.identifiers.org/registry/publisher-id"] ~ [itemprop="value"]'
  return first(selector)?.innerHTML ?? ''
}

const getArticleDoi = (): string => {
  const selector =
    ':--identifier meta[content="https://registry.identifiers.org/registry/doi"] ~ [itemprop="value"]'
  return first(selector)?.innerHTML ?? ''
}

const getArticleTitle = (): string => {
  return first(':--title')?.innerHTML ?? ''
}

ready((): void => {
  formatDate(first(':--datePublished'))

  downloads.build(
    getArticleId(),
    first(':--title')?.getAttribute('content') ?? ''
  )

  references.movePagesEnd(
    references.movePagesStart(
      references.movePeriodicalNames(
        references.moveVolumeNumbers(
          references.moveTitles(select(':--reference'))
        )
      )
    )
  )
  console.log('DOI:', getArticleDoi)
})

// build social media sharers wrapper
// for each of [Facebook, Twitter, Email, Reddit]
//  - build the SVG icon
//  - build icon wrapper
//  - attach icon to icon wrapper
//  - build URL
//  - build anchor element with URL
//  - attach icon wrapper to anchor element
//  - add to social media sharers wrapper

/*
<div class="social-media-sharers">

Example of Facebook URL: https://facebook.com/sharer/sharer.php?u=https%3A%2F%2Fdoi.org%2F10.7554%2FeLife.56026
Needs:
  -  DOI

  <a class="social-media-sharer" href="{{facebookUrl}}" target="_blank" rel="noopener noreferrer" aria-label="Share on Facebook">
    <div class="social-media-sharer__icon_wrapper social-media-sharer__icon_wrapper--facebook social-media-sharer__icon_wrapper--small"><div aria-hidden="true" class="social-media-sharer__icon social-media-sharer__icon--solid">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M18.77 7.46H14.5v-1.9c0-.9.6-1.1 1-1.1h3V.5h-4.33C10.24.5 9.5 3.44 9.5 5.32v2.15h-3v4h3v12h5v-12h3.85l.42-4z"/></svg>
    </div>
    </div>
  </a>

Example of Twitter URL: https://twitter.com/intent/tweet/?text=Centromere%20deletion%20in%20Cryptococcus%20deuterogattii%20leads%20to%20neocentromere%20formation%20and%20chromosome%20fusions&url=https%3A%2F%2Fdoi.org%2F10.7554%2FeLife.56026
Needs:
  - URI-escaped title
  - DOI

<a class="social-media-sharer" href="{{twitterUrl}}" target="_blank" rel="noopener noreferrer" aria-label="Tweet a link to this page">
    <div class="social-media-sharer__icon_wrapper social-media-sharer__icon_wrapper--twitter social-media-sharer__icon_wrapper--small"><div aria-hidden="true" class="social-media-sharer__icon social-media-sharer__icon--solid">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M23.44 4.83c-.8.37-1.5.38-2.22.02.93-.56.98-.96 1.32-2.02-.88.52-1.86.9-2.9 1.1-.82-.88-2-1.43-3.3-1.43-2.5 0-4.55 2.04-4.55 4.54 0 .36.03.7.1 1.04-3.77-.2-7.12-2-9.36-4.75-.4.67-.6 1.45-.6 2.3 0 1.56.8 2.95 2 3.77-.74-.03-1.44-.23-2.05-.57v.06c0 2.2 1.56 4.03 3.64 4.44-.67.2-1.37.2-2.06.08.58 1.8 2.26 3.12 4.25 3.16C5.78 18.1 3.37 18.74 1 18.46c2 1.3 4.4 2.04 6.97 2.04 8.35 0 12.92-6.92 12.92-12.93 0-.2 0-.4-.02-.6.9-.63 1.96-1.22 2.56-2.14z"/></svg>
    </div>
    </div>
  </a>

Example of Email URL: mailto:?subject=Centromere%20deletion%20in%20Cryptococcus%20deuterogattii%20leads%20to%20neocentromere%20formation%20and%20chromosome%20fusions&body=https%3A%2F%2Fdoi.org%2F10.7554%2FeLife.56026
Needs:
  - URI-escaped title
  - DOI

  <a class="social-media-sharer" href="{{emailUrl}}" target="_self" aria-label="Email a link to this page (opens up email program, if configured on this system)">
    <div class="social-media-sharer__icon_wrapper social-media-sharer__icon_wrapper--email social-media-sharer__icon_wrapper--small"><div aria-hidden="true" class="social-media-sharer__icon social-media-sharer__icon--solid">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M22 4H2C.9 4 0 4.9 0 6v12c0 1.1.9 2 2 2h20c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zM7.25 14.43l-3.5 2c-.08.05-.17.07-.25.07-.17 0-.34-.1-.43-.25-.14-.24-.06-.55.18-.68l3.5-2c.24-.14.55-.06.68.18.14.24.06.55-.18.68zm4.75.07c-.1 0-.2-.03-.27-.08l-8.5-5.5c-.23-.15-.3-.46-.15-.7.15-.22.46-.3.7-.14L12 13.4l8.23-5.32c.23-.15.54-.08.7.15.14.23.07.54-.16.7l-8.5 5.5c-.08.04-.17.07-.27.07zm8.93 1.75c-.1.16-.26.25-.43.25-.08 0-.17-.02-.25-.07l-3.5-2c-.24-.13-.32-.44-.18-.68s.44-.32.68-.18l3.5 2c.24.13.32.44.18.68z"/></svg>
    </div>
    </div>
  </a>

Example of Reddit URL: https://reddit.com/submit/?title=Centromere%20deletion%20in%20Cryptococcus%20deuterogattii%20leads%20to%20neocentromere%20formation%20and%20chromosome%20fusions&url=https%3A%2F%2Fdoi.org%2F10.7554%2FeLife.56026
Needs:
  - URI-escaped title
  - DOI

  <a class="social-media-sharer" href="{{redditUrl}}" target="_blank" rel="noopener noreferrer" aria-label="Share this page on Reddit">
    <div class="social-media-sharer__icon_wrapper social-media-sharer__icon_wrapper--reddit social-media-sharer__icon_wrapper--small"><div aria-hidden="true" class="social-media-sharer__icon social-media-sharer__icon--solid">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M24 11.5c0-1.65-1.35-3-3-3-.96 0-1.86.48-2.42 1.24-1.64-1-3.75-1.64-6.07-1.72.08-1.1.4-3.05 1.52-3.7.72-.4 1.73-.24 3 .5C17.2 6.3 18.46 7.5 20 7.5c1.65 0 3-1.35 3-3s-1.35-3-3-3c-1.38 0-2.54.94-2.88 2.22-1.43-.72-2.64-.8-3.6-.25-1.64.94-1.95 3.47-2 4.55-2.33.08-4.45.7-6.1 1.72C4.86 8.98 3.96 8.5 3 8.5c-1.65 0-3 1.35-3 3 0 1.32.84 2.44 2.05 2.84-.03.22-.05.44-.05.66 0 3.86 4.5 7 10 7s10-3.14 10-7c0-.22-.02-.44-.05-.66 1.2-.4 2.05-1.54 2.05-2.84zM2.3 13.37C1.5 13.07 1 12.35 1 11.5c0-1.1.9-2 2-2 .64 0 1.22.32 1.6.82-1.1.85-1.92 1.9-2.3 3.05zm3.7.13c0-1.1.9-2 2-2s2 .9 2 2-.9 2-2 2-2-.9-2-2zm9.8 4.8c-1.08.63-2.42.96-3.8.96-1.4 0-2.74-.34-3.8-.95-.24-.13-.32-.44-.2-.68.15-.24.46-.32.7-.18 1.83 1.06 4.76 1.06 6.6 0 .23-.13.53-.05.67.2.14.23.06.54-.18.67zm.2-2.8c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2zm5.7-2.13c-.38-1.16-1.2-2.2-2.3-3.05.38-.5.97-.82 1.6-.82 1.1 0 2 .9 2 2 0 .84-.53 1.57-1.3 1.87z"/></svg>
    </div>
    </div>
  </a>

</div>
*/
