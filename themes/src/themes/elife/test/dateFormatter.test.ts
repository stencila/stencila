import * as dateFormatter from '../lib/dateFormatter'

describe('dateFormatter', () => {
  describe('format', () => {
    it('returns the correct date format', () => {
      const dates = [
        {
          value: '2020-05-06',
          expectedFormat: 'May 6, 2020',
        },
        {
          value: '2019-09-30',
          expectedFormat: 'Sep 30, 2019',
        },
      ]
      dates.forEach((date: { value: string; expectedFormat: string }) => {
        const element = document.createElement('span')
        element.innerHTML = date.value
        dateFormatter.format(element)
        expect(element.innerHTML).toEqual(date.expectedFormat)
      })
    })

    it('throws a RangeError if an invalid date is passed', () => {
      const element = document.createElement('span')
      element.innerHTML = '2020-05-32'
      expect(() => {
        dateFormatter.format(element)
      }).toThrowError(RangeError)
    })
  })
})
