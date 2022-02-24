import { customAlphabet } from 'nanoid'

/**
 * The epoch used for calculating
 * the time stamp. Chosen as a recent time
 * that was easily remembered.
 * Happens to correspond to 2017-07-14T02:40:00.000Z.
 */
export const epoch = 1500000000000

/**
 * A random id generator using custom alphabet and length.
 */
const nanoid = customAlphabet(
  '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz',
  20
)

/**
 * Generate a unique identifier.
 *
 * Generated identifiers:
 *
 * - are URL safe
 * - contain information on the type of object that they
 *   are identifying
 * - are roughly sortable by time of generation
 * - have an extremely low probability of collision
 *
 * Generated identifiers have a fixed length of 32 characters made up
 * of three parts separated by dots:
 *
 * - 2 characters in the range `[a-z]` that identifying the "family" of
 *   identifiers, usually the type of object
 *   the identifier is for e.g. `rq` = request
 *
 * - 10 characters in the range `[0-9a-f]` that are the hexadecimal encoding of the
 *   seconds since `2017-07-14T02:40:00.000Z`the hexadecimal
 *
 * - 20 characters in the range `[0-9A-Za-z]` that are randomly generated
 *
 * @see {@link https://segment.com/blog/a-brief-history-of-the-uuid/|A brief history of the UUID}
 * @see {@link https://zelark.github.io/nano-id-cc/|Nano ID Collision Calculator}
 */
export function generate(code: string): { family: string; value: string } {
  const family =
    code.length === 2
      ? code
      : code.length === 0
      ? 'uu'
      : code.length === 1
      ? code.repeat(2)
      : code.slice(0, 2)
  const time = (Date.now() - epoch).toString(16).padStart(10, '0')
  const rand = nanoid()
  const value = `${family}.${time}.${rand}`
  return { family, value }
}

/**
 * Parse a unique identifier.
 *
 * Extracts the parts from the identifier: `family`, `time` and `rand`.
 */
export function parse(id: string):
  | {
      family: string
      time: Date
      rand: string
    }
  | undefined {
  const [_, family, t, rand] =
    /^([a-z]{2})\.([0-9a-f]{8})\.([0-9A-Za-z]{20})$/.exec(id) ?? []

  if (!family || !t || !rand) return

  let seconds
  try {
    seconds = parseInt(t ?? '', 16)
  } catch (error) {
    return undefined
  }

  const time = new Date(seconds + epoch)
  return { family, time, rand }
}
