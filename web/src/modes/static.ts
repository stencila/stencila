import '../themes/base'

import { waitForElems } from '../utils/curtain'

import '../components/document/document-header'
import '../components/document/document-footer'
import '../components/document/document-nav'
import '../components/document/document-toc'

waitForElems([
  'document-header',
  'document-footer',
  'document-nav',
  'document-toc',
])
