/**
 * Script to imports [Citation Typing Ontology (CiTO)](http://www.sparontologies.net/ontologies/cito)
 * citation types as the `CitationTypeEnumeration`.
 *
 * Fetches the CiTO JSON spec from GitHub and converts it into a type that extends `Enumeration`.
 * The names of citation types are all made PascalCase, linked to their CiTo `@id`, and description
 * copied over.
 *
 * Excludes CiTO types `Citation` and `cites` since these are already implicit in the `stencila:Cite`
 * type and are not a type of citation. Excludes `Likes` because it does not represent a formal citation type:
 * "Use of this property does NOT imply the existence of a formal citation of the entity that is 'liked'."
 *
 * Excludes `hasCitationCharacterization`, `hasCitationTimeSpan`, etc since these describe properties
 * of a citation other than it's type.
 *
 * Excludes `*Citation` and `Shares*` since these are related to citation "distance", not citation
 * intent. See https://github.com/stencila/schema/pull/260#pullrequestreview-636331806.
 *
 * Run using `npx ts-node ts/imports/cito.ts`.
 */

import got from 'got'
import fs from 'fs-extra'
import path from 'path'
import { pascalCase, sentenceCase } from 'change-case'

// eslint-disable-next-line @typescript-eslint/no-floating-promises
;(async () => {
  const {
    body,
  } = await got.get(
    'https://raw.githubusercontent.com/SPAROntologies/cito/b4c5b10a8d7b5f0da6ea8a4f3edcc00d7984f9a4/docs/current/cito.json',
    { responseType: 'json' }
  )

  const citoUrl = 'http://purl.org/spar/cito/'
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const types = (body as Array<Record<string, any>>).reduce(
    (prev: Record<string, string>, type) => {
      const idUrl = type['@id'] as string

      if (!idUrl.startsWith(citoUrl)) return prev
      const id = idUrl.replace(citoUrl, '')

      if (
        [
          'Citation',
          'cites',
          'hasCitationCharacterization',
          'hasCitationCreationDate',
          'hasCitationTimeSpan',
          'hasCitedEntity',
          'hasCitingEntity',
          'hasCoAuthorshipCitationLevel',
          'Likes',
        ].includes(id) ||
        id.startsWith('Shares') ||
        id.endsWith('Citation')
      )
        return prev

      const title = pascalCase(id)
      const rdfComment =
        // eslint-disable-next-line
        type['http://www.w3.org/2000/01/rdf-schema#comment']?.[0]?.['@value']
      const description =
        typeof rdfComment === 'string'
          ? rdfComment.split('.')[0]
          : sentenceCase(id)

      const yaml = `
  - const: ${title}
    '@id': cito:${id}
    description: ${description}
`
      return { ...prev, [title]: yaml }
    },
    {}
  )

  const yaml = Object.entries(types)
    .sort()
    .map(([_title, yaml]) => yaml)
    .join('')

  fs.writeFileSync(
    path.join(
      __dirname,
      '..',
      '..',
      'schema',
      'CitationTypeEnumeration.schema.yaml'
    ),
    `title: CitationTypeEnumeration
'@id': stencila:CitationTypeEnumeration
status: unstable
role: secondary
category: metadata
extends: Enumeration
description: The type or nature of a citation, both factually and rhetorically.
$comment: |
  The members of this enumeration map directly on to the types in the [Citation Typing Ontology (CiTO)](http://www.sparontologies.net/ontologies/cito).
anyOf:${yaml}`
  )
})()
