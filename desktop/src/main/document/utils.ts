import { FileFormatUtils } from '@stencila/components'
import { FileFilter } from 'electron'
import { array as A, eq, ord, string as S } from 'fp-ts'
import { pipe } from 'fp-ts/function'

const encodaFormats: FileFormatUtils.FileFormatMap = {
  DOCX: {
    name: 'Microsoft Word',
    ext: 'docx',
    aliases: [],
  },
  Excel: {
    name: 'Microsoft Excel',
    ext: 'xsls',
    aliases: [],
  },
  GDocs: {
    name: 'Google Docs',
    ext: 'gdoc',
    aliases: [],
  },
  JATS: {
    name: 'JATS XML',
    ext: 'jats.xml',
    aliases: [],
  },
  PDF: {
    name: 'PDF',
    ext: 'pdf',
    aliases: [],
  },
  MathML: { name: 'MathML', ext: 'mathml', aliases: [] },
  CSL: {
    name: 'CSL (Citation Style Language JSON)',
    ext: 'csl',
    aliases: [],
  },
  BibTeX: {
    name: 'BibTeX',
    ext: 'bib',
    aliases: [],
  },
  CSV: {
    name: 'CSV',
    ext: 'csv',
    aliases: [],
  },
  TDP: {
    name: 'TDP (Tabular Data Package)',
    ext: 'tdp',
    aliases: [],
  },
}

const ordByName = pipe(
  S.Ord,
  ord.contramap((f: FileFormatUtils.FileFormat) => f.name)
)

const eqByName = pipe(
  S.Ord,
  eq.contramap((f: FileFormatUtils.FileFormat) => f.name)
)

/**
 * Convert `FileFormat` object to an Electron compatible `FileFilter`
 */
const fileFormatToFileFilter = ({
  name,
  ext,
  aliases,
}: FileFormatUtils.FileFormat): FileFilter => ({
  name,
  extensions: [ext ?? '', ...aliases],
})

/**
 * Combined mapping of all supported file formats.
 * This includes both the Code Editor supported syntaxes, as well as formats supported by Encoda.
 */
const allFormats = Object.values({
  default: {
    name: '',
    ext: '*',
    aliases: [],
  },
  ...FileFormatUtils.fileFormatMap,
  ...encodaFormats,
})

export const supportedFileFormats = pipe(
  allFormats,
  A.sort(ordByName),
  A.uniq(eqByName),
  A.reduce([], (filters: FileFilter[], format) => [
    ...filters,
    fileFormatToFileFilter(format),
  ])
)
