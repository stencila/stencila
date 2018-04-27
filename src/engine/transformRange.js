import { isNumber } from 'substance'

export default function transformRange(start, end, pos, count) {
  if (!count) return false
  if(!isNumber(pos) || !isNumber(count)) throw new Error("pos and count must be integers")
  if(end < pos) return false
  if(count > 0) {
    if(pos <= start) {
      start += count
    }
    if(pos <= end) {
      end += count
    }
  } else {
    // for removal count < 0
    count = -count
    // null means deleted
    if (start >= pos && end < pos + count) return null
    const x1 = pos
    const x2 = pos + count
    if (x2 <= start) {
      start -= count
      end -= count
    } else {
      if (pos <= start) {
        start = start - Math.min(count, start-x1)
      }
      if (pos <= end) {
        end = end - Math.min(count, end-x1+1)
      }
    }
  }
  return { start, end }
}
