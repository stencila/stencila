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
  expect_equal(class(Array(character)), "Array")

  expect_equal(format(Array(numeric)), "Array(numeric)")
  expect_equal(format(Array(Union(character, Person))), "Array(Union(character, Person))")

  expect_true(is_type(vector("numeric"), Array(numeric)))
  expect_true(is_type(vector("numeric"), Array("numeric"))) #Alternative syntax
  expect_true(is_type(vector("logical"), Array(logical)))
  expect_true(is_type(1, Array(numeric)))
  expect_true(is_type(1:10, Array(numeric)))
  expect_true(is_type(list(1), Array(numeric)))
  expect_true(is_type(list(1, 2, 3), Array(numeric)))
  expect_true(is_type(list(Person(), Person()), Array(Person)))
  expect_true(is_type(list("abc", Person()), Array(Union(character, Person))))

  expect_false(is_type(NULL, Array(numeric)))
  expect_false(is_type(NA, Array(numeric)))
  expect_false(is_type(Thing(), Array(numeric)))
  expect_false(is_type(Person(), Array(numeric)))
})

test_that("Union", {
  expect_equal(class(Union(character)), "Union")

  expect_equal(format(Union(numeric, character)), "Union(numeric, character)")
  expect_equal(format(Union("numeric", "character")), "Union(numeric, character)") # Alternative syntax
  expect_equal(format(Union(character, Person)), "Union(character, Person)")

  expect_true(is_type(1, Union(numeric, character)))
  expect_true(is_type("string", Union(numeric, character)))
  expect_true(is_type(Person(), Union(character, Person)))

  expect_false(is_type(NULL, Union(numeric)))
  expect_false(is_type(NA, Union(numeric)))
  expect_false(is_type(Person(), Union(numeric)))
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

test_that("mode_to_schema_type", {
  expect_equal(mode_to_schema_type("logical"), "boolean")
  expect_equal(mode_to_schema_type("numeric"), "number")
  expect_equal(mode_to_schema_type("character"), "string")
  expect_equal(mode_to_schema_type("list"), "object")
})

test_that("schema_type_to_mode", {
  expect_equal(schema_type_to_mode("boolean"), "logical")
  expect_equal(schema_type_to_mode("number"), "numeric")
  expect_equal(schema_type_to_mode("string"), "character")
  expect_equal(schema_type_to_mode("object"), "list")
})

test_that("is_type", {
  expect_false(is_type(list(1, 2, 3), Array("character")))
  expect_true(is_type(list(1, 2, 3), Array("numeric")))

  expect_false(is_type(factor(1:10), Array("numeric")))
  expect_true(is_type(factor(1:10), Array("character")))

  p <- Paragraph(list(""))
  expect_true(is_type(p, Node))
  expect_true(is_type(p, Union(numeric, Node)))
  expect_true(is_type(p, Paragraph))
  expect_true(is_type(p, BlockContent))
  expect_false(is_type(p, InlineContent))
  expect_true(is_type(p, Union(InlineContent, BlockContent)))
})

test_that("assert_type", {
  assert_type(NULL, "NULL")
  assert_type(1, "numeric")
  assert_type("string", "character")
  assert_type(Person(), "Person")

  expect_error(assert_type(Person(), "numeric"), "value is type Person, expected type numeric")
})

test_that("check_property", {
  expect_equal(
    check_property(
      type_name = "type",
      property_name = "property",
      is_required = FALSE,
      is_missing = TRUE,
      type = "character",
      value = "foo"
    ),
    NULL
  )

  expect_equal(
    class(check_property(
      type_name = "type",
      property_name = "property",
      is_required = FALSE,
      is_missing = FALSE,
      type = "character",
      value = "foo"
    )),
    c("scalar", "character")
  )

  expect_equal(
    class(check_property(
      type_name = "type",
      property_name = "property",
      is_required = FALSE,
      is_missing = FALSE,
      type = Array("character"),
      value = "foo"
    )),
    c("character")
  )

  expect_error(
    check_property(
      type_name = "type",
      property_name = "property",
      is_required = TRUE,
      is_missing = TRUE,
      type = "character",
      value = "foo"
    ),
    "type\\$property is required"
  )

  expect_error(
    check_property(
      type_name = "type",
      property_name = "property",
      is_required = TRUE,
      is_missing = FALSE,
      type = "character",
      value = 42
    ),
    "type\\$property is type numeric, expected type character"
  )
})
