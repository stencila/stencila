import { Date, Number, Text, URL } from './dataTypes'
import Organization from './Organization'
import Person from './Person'
import Thing from './Thing'

/**
 * The most generic kind of creative work, including books, movies, photographs, software programs, etc.
 * https://schema.org/CreativeWork
 */
export default class CreativeWork extends Thing {
  authors: Array<Organization | Person> = []

  contributors: Array<Organization | Person> = []

  creators: Array<Organization | Person> = []

  datePublished: Date = ''

  license: CreativeWork | URL = ''

  text: Text = ''

  version: Number | Text = ''
}
