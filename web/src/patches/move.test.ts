import { applyMove, applyMoveVec } from './move'

test('applyMoveVec', () => {
  const elem = document.createElement('ol')
  elem.innerHTML = '<li>1</li><li>2</li><li>3</li><li>4</li><li>5</li>'

  applyMoveVec(elem, 0, 1, 1)
  expect(elem.innerHTML).toEqual(
    '<li>2</li><li>1</li><li>3</li><li>4</li><li>5</li>'
  )

  applyMoveVec(elem, 0, 5, 2)
  expect(elem.innerHTML).toEqual(
    '<li>3</li><li>4</li><li>5</li><li>2</li><li>1</li>'
  )

  applyMoveVec(elem, 4, 0, 1)
  expect(elem.innerHTML).toEqual(
    '<li>1</li><li>3</li><li>4</li><li>5</li><li>2</li>'
  )

  applyMoveVec(elem, 4, 1, 1)
  expect(elem.innerHTML).toEqual(
    '<li>1</li><li>2</li><li>3</li><li>4</li><li>5</li>'
  )

  expect(() => applyMoveVec(elem, 'string', 1, 1)).toThrow(
    /Expected number slot/
  )
  expect(() => applyMoveVec(elem, -1, 1, 1)).toThrow(
    /Unexpected move from slot -1/
  )
  expect(() => applyMoveVec(elem, 0, 100, 1)).toThrow(
    /Unexpected move to slot 100/
  )
  expect(() => applyMoveVec(elem, 0, 1, 100)).toThrow(
    /Unexpected move items 100/
  )
})
