import { Date, Number, Text, URL } from './dataTypes'
import { type, property } from './decorators'
import Organization from './Organization'
import Person from './Person'
import Thing from './Thing'

/**
 * The most generic kind of creative work, including books, movies, photographs, software programs, etc.
 */
@type('schema:CreativeWork')
export default class CreativeWork extends Thing {

  @property('schema:author', 'list')
  authors: Array<Organization | Person> = []

  @property('schema:contributor', 'list')
  contributors: Array<Organization | Person> = []

  @property('schema:creator', 'list')
  creators: Array<Organization | Person> = []

  @property('schema:datePublished')
  datePublished: Date = ''

  @property('schema:license')
  license: CreativeWork | URL = ''

  @property('schema:text')
  text: Text = ''

  @property('schema:version')
  version: Number | Text = ''
}
