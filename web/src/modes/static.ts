import '../themes/base'

import '../components/document/document-header'
import '../components/document/document-footer'
import '../components/document/document-nav'
import '../components/document/document-toc'

import { waitForElems } from '../utils/curtain'
waitForElems([
  'document-header',
  'document-footer',
  'document-nav',
  'document-toc',
])
