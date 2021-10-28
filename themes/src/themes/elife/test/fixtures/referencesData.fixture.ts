import { create } from '../../../../util'

/* Using this canned data is not great as it's a snapshot in time of how the reference comes over from
   Encoda, which may change. A possible future improvement could be to read the file
     /src/examples/articleReplication.html
   and use the reference list within as the data source for the unformatted references.
   This would make sure that the tests are run against the current markup and so stay
   relevant. It would also run the tests against different types of reference, making them
   more useful at identifying gaps in the formatting, and any regressions that might
   creep in.
 */
// Reference 5 from articleReplication.html example.
const unformattedReference =
  '<li itemscope="" itemtype="http://schema.org/Article" itemprop="citation" id="bib5">\n' +
  '        <ol data-itemprop="authors">\n' +
  '          <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">\n' +
  '            <meta itemprop="name" content="DJ Duffy"><span data-itemprop="givenNames"><span itemprop="givenName">DJ</span></span><span data-itemprop="familyNames"><span itemprop="familyName">Duffy</span></span></li>\n' +
  '          <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">\n' +
  '            <meta itemprop="name" content="A Krstic"><span data-itemprop="givenNames"><span itemprop="givenName">A</span></span><span data-itemprop="familyNames"><span itemprop="familyName">Krstic</span></span></li>\n' +
  '          <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">\n' +
  '            <meta itemprop="name" content="M Halasz"><span data-itemprop="givenNames"><span itemprop="givenName">M</span></span><span data-itemprop="familyNames"><span itemprop="familyName">Halasz</span></span></li>\n' +
  '          <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">\n' +
  '            <meta itemprop="name" content="T Schwarzl"><span data-itemprop="givenNames"><span itemprop="givenName">T</span></span><span data-itemprop="familyNames"><span itemprop="familyName">Schwarzl</span></span></li>\n' +
  '          <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">\n' +
  '            <meta itemprop="name" content="D Fey"><span data-itemprop="givenNames"><span itemprop="givenName">D</span></span><span data-itemprop="familyNames"><span itemprop="familyName">Fey</span></span></li>\n' +
  '          <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">\n' +
  '            <meta itemprop="name" content="K Iljin"><span data-itemprop="givenNames"><span itemprop="givenName">K</span></span><span data-itemprop="familyNames"><span itemprop="familyName">Iljin</span></span></li>\n' +
  '          <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">\n' +
  '            <meta itemprop="name" content="JP Mehta"><span data-itemprop="givenNames"><span itemprop="givenName">JP</span></span><span data-itemprop="familyNames"><span itemprop="familyName">Mehta</span></span></li>\n' +
  '          <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">\n' +
  '            <meta itemprop="name" content="K Killick"><span data-itemprop="givenNames"><span itemprop="givenName">K</span></span><span data-itemprop="familyNames"><span itemprop="familyName">Killick</span></span></li>\n' +
  '          <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">\n' +
  '            <meta itemprop="name" content="J Whilde"><span data-itemprop="givenNames"><span itemprop="givenName">J</span></span><span data-itemprop="familyNames"><span itemprop="familyName">Whilde</span></span></li>\n' +
  '          <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">\n' +
  '            <meta itemprop="name" content="B Turriziani"><span data-itemprop="givenNames"><span itemprop="givenName">B</span></span><span data-itemprop="familyNames"><span itemprop="familyName">Turriziani</span></span></li>\n' +
  '          <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">\n' +
  '            <meta itemprop="name" content="S Haapa-Paananen"><span data-itemprop="givenNames"><span itemprop="givenName">S</span></span><span data-itemprop="familyNames"><span itemprop="familyName">Haapa-Paananen</span></span></li>\n' +
  '          <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">\n' +
  '            <meta itemprop="name" content="V Fey"><span data-itemprop="givenNames"><span itemprop="givenName">V</span></span><span data-itemprop="familyNames"><span itemprop="familyName">Fey</span></span></li>\n' +
  '          <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">\n' +
  '            <meta itemprop="name" content="M Fischer"><span data-itemprop="givenNames"><span itemprop="givenName">M</span></span><span data-itemprop="familyNames"><span itemprop="familyName">Fischer</span></span></li>\n' +
  '          <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">\n' +
  '            <meta itemprop="name" content="F Westermann"><span data-itemprop="givenNames"><span itemprop="givenName">F</span></span><span data-itemprop="familyNames"><span itemprop="familyName">Westermann</span></span></li>\n' +
  '          <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">\n' +
  '            <meta itemprop="name" content="K-O Henrich"><span data-itemprop="givenNames"><span itemprop="givenName">K-O</span></span><span data-itemprop="familyNames"><span itemprop="familyName">Henrich</span></span></li>\n' +
  '          <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">\n' +
  '            <meta itemprop="name" content="S Bannert"><span data-itemprop="givenNames"><span itemprop="givenName">S</span></span><span data-itemprop="familyNames"><span itemprop="familyName">Bannert</span></span></li>\n' +
  '          <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">\n' +
  '            <meta itemprop="name" content="DG Higgins"><span data-itemprop="givenNames"><span itemprop="givenName">DG</span></span><span data-itemprop="familyNames"><span itemprop="familyName">Higgins</span></span></li>\n' +
  '          <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">\n' +
  '            <meta itemprop="name" content="W Kolch"><span data-itemprop="givenNames"><span itemprop="givenName">W</span></span><span data-itemprop="familyNames"><span itemprop="familyName">Kolch</span></span></li>\n' +
  '        </ol><time itemprop="datePublished" datetime="2015">2015</time><span itemprop="headline" content="Integrative omics reveals MYCN as a global suppressor of cellular signalling and enables network-based therapeutic target discovery in neuroblastoma">Integrative\n' +
  '          omics reveals MYCN as a global suppressor of cellular signalling and enables network-based\n' +
  '          therapeutic target discovery in neuroblastoma</span><span itemscope="" itemtype="http://schema.org/PublicationVolume" itemprop="isPartOf"><span itemprop="volumeNumber" data-itemtype="http://schema.org/Number">6</span><span itemscope="" itemtype="http://schema.org/Periodical" itemprop="isPartOf"><span itemprop="name">Oncotarget</span></span></span><span itemprop="pageStart" data-itemtype="http://schema.org/Number">43182</span><span itemprop="pageEnd" data-itemtype="http://schema.org/Number">43201</span><span itemscope="" itemtype="http://schema.org/Organization" itemprop="publisher">\n' +
  '          <meta itemprop="name" content="Unknown"><span itemscope="" itemtype="http://schema.org/ImageObject" itemprop="logo">\n' +
  '            <meta itemprop="url" content="https://via.placeholder.com/600x60/dbdbdb/4a4a4a.png?text=Unknown">\n' +
  '            </span></span>\n' +
  '        <meta itemprop="image" content="https://via.placeholder.com/1200x714/dbdbdb/4a4a4a.png?text=Integrative%20omics%20reveals%20MYCN%20as%20a%20global%20suppressor%20of%20cellular%20signalling%20and%20enables%20network-based%20therap%E2%80%A6">\n' +
  '      </li>'
export const getFixtureData = (): Element => {
  return create('ol', null, create(unformattedReference))
}
