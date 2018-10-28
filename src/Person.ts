import { Text } from './dataTypes'
import { type, property } from './decorators'
import Thing from './Thing'

@type('schema:Person')
export default class Person extends Thing {

  @property('schema:email', 'list')
  emails: Array<Text> = []

  @property('schema:familyName', 'list')
  familyNames: Array<Text> = []

  @property('schema:givenName', 'list')
  givenNames: Array<Text> = []

  /**
   * Create a `Person` object from a `Text` value.
   *
   * The text value can contain email and URL in the format:
   *
   *   Joe Bloggs <joe@example.com> (https://example.com/joe)
   *
   * @param text The text value to parse
   * @returns A `Person` object
   */
  static fromText (text: Text): Person {
    const person = new Person()
    const match = text.match(/^(?:\s*)([^\s]*)(?:\s+)?([^\s]+)?\s*(<([^>]*)>)?\s*(\(([^)]*)\))?/)
    if (match) {
      if (match[1]) {
        person.givenNames = [match[1]]
        person.name = person.givenNames.join(' ')
      }
      if (match[2]) {
        person.familyNames = [match[2]]
        person.name += ' ' + person.familyNames.join(' ')
      }
      if (match[4]) person.emails = [match[4]]
      if (match[6]) person.urls = [match[6]]
    } else {
      person.name = text
    }
    return person
  }
}
