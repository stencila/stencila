context("typing")

test_that("Any", {
  expect_equal(class(Any()), "Any")

  expect_equal(format(Any()), "Any()")

  expect_true(is_type(NULL, Any()))
  expect_true(is_type(NA, Any()))
  expect_true(is_type(1, Any()))
  expect_true(is_type(Thing(), Any()))
  expect_true(is_type(Article(), Any()))
})

test_that("Array", {
  expect_equal(class(Array("character")), "Array")

  expect_equal(format(Array("numeric")), "Array(numeric)")
  expect_equal(format(Array(Union("string", "Person"))), "Array(Union(string, Person))")

  expect_true(is_type(vector("numeric"), Array("numeric")))
  expect_true(is_type(vector("logical"), Array("logical")))
  expect_true(is_type(1, Array("numeric")))
  expect_true(is_type(1:10, Array("numeric")))
  expect_true(is_type(list(1), Array("numeric")))
  expect_true(is_type(list(1, 2, 3), Array("numeric")))
  expect_true(is_type(list(Person(), Person()), Array("Person")))
  expect_true(is_type(list("abc", Person()), Array(Union("character", "Person"))))

  expect_false(is_type(NULL, Array("numeric")))
  expect_false(is_type(NA, Array("numeric")))
  expect_false(is_type(Thing(), Array("numeric")))
  expect_false(is_type(Person(), Array("numeric")))
})

test_that("Union", {
  expect_equal(class(Union("character")), "Union")

  expect_equal(format(Union("numeric", "character")), "Union(numeric, character)")
  expect_equal(format(Union("character", "Person")), "Union(character, Person)")

  expect_true(is_type(1, Union("numeric", "character")))
  expect_true(is_type("string", Union("numeric", "character")))
  expect_true(is_type(Person(), Union("string", "Person")))

  expect_false(is_type(NULL, Union("numeric")))
  expect_false(is_type(NA, Union("numeric")))
  expect_false(is_type(Person(), Union("numeric")))
})


test_that("Enum", {
  enum <- Enum("a", "b", "c")

  expect_equal(class(enum), "Enum")

  expect_equal(format(enum), "Enum(a, b, c)")

  expect_true(is_type("a", enum))
  expect_true(is_type("c", enum))

  expect_false(is_type("d", enum))
  expect_false(is_type(NA, enum))
  expect_false(is_type(1, enum))
  expect_false(is_type(Person(), enum))
})

test_that("assert_type", {
  assert_type(NULL, "NULL")
  assert_type(1, "numeric")
  assert_type("string", "character")
  assert_type(Person(), "Person")

  expect_error(assert_type(Person(), "numeric"), "value is type Person, expected type numeric")
})
