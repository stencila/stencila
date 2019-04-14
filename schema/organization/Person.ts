//@ts-ignore
import parseAuthor from 'parse-author'
//@ts-ignore
import { parseFullName } from 'parse-full-name'
import { Person } from '../../types'
import { validate } from '../../util'

export default Person

/**
 * Parse string data into a `Person`.
 *
 * @param data Data to parse
 */
export function parse(data: string): Person {
  const { name, email, url } = parseAuthor(data)
  const { title, first, middle, last, suffix } = parseFullName(name)
  const person: Person = { type: 'Person' }
  if (title) person.honorificPrefix = title
  if (first) {
    person.givenNames = [first]
    if (middle) person.givenNames.push(middle)
  }
  if (last) person.familyNames = [last]
  else throw new Error(`Unable to parse string "${data}" as a person`)
  if (suffix) person.honorificSuffix = suffix
  if (email) person.emails = [email]
  if (url) person.url = url
  validate(person, 'Person')
  return person
}
