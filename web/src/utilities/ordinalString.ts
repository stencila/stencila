/**
 * Returns the ordinal string for the number provided,
 * eg '1st', '3rd', '113th' etc...
 */
export const ordinalString = (num: number): string => {
  const x = num % 10,
    y = num % 100

  if (x === 1 && y !== 11) {
    return `${num}st`
  }
  if (x === 2 && y !== 12) {
    return `${num}nd`
  }
  if (x === 3 && y !== 13) {
    return `${num}rd`
  }
  return `${num}th`
}
