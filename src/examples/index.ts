import { examples } from './examples'

export { examples }

/**
 * Is the string an example name?
 *
 * @param {string} name Name of the example
 */
export const isExample = (name: string): name is keyof typeof examples =>
  name in examples

/**
 * Given a string, will return a matching example,
 * falling back to the first in if none matches.
 *
 * @param {string} name Name of the example to look for
 */
export const resolveExample = (name?: string): string => {
  const example = name === undefined ? '' : name.trim()
  return isExample(example) ? examples[example] : examples.articleKitchenSink
}
