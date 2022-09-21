import { customAlphabet } from 'nanoid'

const idGenerator: Record<number, () => string> = {}

/**
 * A random id generator using custom alphabet and length.
 * Because instantiating the function can be expensive, we cache the generators
 * based on the size of the ID required.
 */
const nanoid = (size = 20): string => {
  let uidGenerator = idGenerator[size]

  if (uidGenerator !== undefined) {
    return uidGenerator()
  } else {
    uidGenerator = customAlphabet(
      '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz',
      size
    )

    idGenerator[size] = uidGenerator
    return uidGenerator()
  }
}

/**
 * Generate a unique identifier.
 *
 * Generated identifiers:
 *
 * - are URL safe
 * - contain information on the type of object that they
 *   are identifying
 * - have an extremely low probability of collision
 *
 * Generated identifiers have a fixed length of 32 characters made up
 * of three parts separated by dots:
 *
 * - 2 characters in the range `[a-z]` that identifying the "family" of
 *   identifiers, usually the type of object the identifier is for:
 *   - do: document
 *   - cb: code block
 *   - cc: code chunk
 *   - ce: code expression
 *   - cf: code fragment
 *   - mb: math block
 *   - mf: math fragment
 *   - re: request
 *   - cl: client
 *   - su: subscription
 *   - ta: task
 *   - ke: kernel
 *   - se: session
 *
 * - 20 characters in the range `[0-9A-Za-z]` that are randomly generated.
 *   The length can be adjusted by the `size` parameter.
 *
 * @see {@link https://segment.com/blog/a-brief-history-of-the-uuid/|A brief history of the UUID}
 * @see {@link https://zelark.github.io/nano-id-cc/ |Nano ID Collision Calculator}
 */
export const generate = (code: string, size?: number): string => {
  const family =
    code.length === 2
      ? code
      : code.length === 0
      ? 'uu'
      : code.length === 1
      ? code.repeat(2)
      : code.slice(0, 2)

  const rand = nanoid(size)
  return `${family}-${rand}`
}
