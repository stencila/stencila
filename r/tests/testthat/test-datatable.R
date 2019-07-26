context("datatable")

test_that("from_dataframe + to_dataframe", {
  mtcars_dt <- from_dataframe(mtcars)

  expect_equal(length(mtcars_dt$columns), 11)
  expect_equal(
    sapply(mtcars_dt$columns, function(column) column$name),
    c("mpg", "cyl", "disp", "hp", "drat", "wt", "qsec", "vs", "am", "gear", "carb")
  )
  expect_equal(
    sapply(mtcars_dt$columns, function(column) column$schema$items$type),
    c("number", "number", "number", "number", "number", "number", "number", "number", "number", "number", "number")
  )
})
