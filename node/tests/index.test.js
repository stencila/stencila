const { sum } = require("../index.js");

test("sum from native", () => {
  expect(sum(1, 2)).toBe(3);
});
