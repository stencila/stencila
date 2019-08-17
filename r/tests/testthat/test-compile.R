context("compile")

test_that("compile_code_chunk: imports", {
  imports <- function(text) compile_code_chunk(text)$imports

  expect_equal(
    imports("#' @imports pkg"),
    "pkg"
  )

  expect_equal(
    imports("# @imports pkg1, pkg2"),
    c("pkg1", "pkg2")
  )

  expect_equal(
    imports("foo # @imports pkg"),
    "pkg"
  )

  expect_equal(
    imports("# @imports pkg1\n# @imports pkg2, pkg3\nfoo #@imports pkg4"),
    c("pkg1", "pkg2", "pkg3", "pkg4")
  )

  expect_equal(
    imports("library(pkg)"),
    "pkg"
  )

  expect_equal(
    imports("library('pkg1'); require(pkg2)"),
    c("pkg1", "pkg2")
  )

  # Unquoted package names should not appear in `uses`
  chunk <- compile_code_chunk("library(pkg1)\n# @imports pkg2\nrequire(pkg3)")
  expect_equal(
    chunk$imports,
    c("pkg2", "pkg1", "pkg3")
  )
  expect_equal(
    chunk$uses,
    NULL
  )
})

test_that("compile_code_chunk: declares", {
  declares <- function(text) compile_code_chunk(text)$declares

  expect_equal(
    declares("func <- function(){}"),
    list(
      Function(
        name = "func"
      )
    )
  )

  expect_equal(
    declares("func1 <- function(a){}\nfunc2 <- function(a, b){}"),
    list(
      Function(
        name = "func1",
        parameters = list(
          Parameter("a")
        )
      ),
      Function(
        name = "func2",
        parameters = list(
          Parameter("a"),
          Parameter("b")
        )
      )
    )
  )

  # Function declarations should not appear in `uses` unless
  # used before declaration.
  chunk <- compile_code_chunk(paste(
    "func2()",
    "func1 <- function(){}",
    "func1()",
    "func2 <- function(){}",
    sep = "\n"
  ))
  expect_equal(
    sapply(chunk$declares, function(func) func$name),
    c("func1", "func2")
  )
  expect_equal(
    chunk$uses,
    "func2"
  )
})

test_that("compile_code_chunk: assigns", {
  assigns <- function(text) compile_code_chunk(text)$assigns

  # The usual top level assignments
  expect_equal(
    assigns(paste(
      "var1 <- 1",
      "var2 = 2",
      "assign('var3', 3)",
      "# @assigns var4, var5",
      "var6 <<- 6",
      sep = "\n"
    )),
    c("var4", "var5", "var1", "var2", "var3", "var6")
  )

  # Assignments in functions are ignored
  # except global assigns <<- and assigns function
  expect_equal(
    assigns(paste(
      "func1 <- function() {",
      "  a <- 1",
      "  b = 2",
      "  c <<- 3",
      "  assign('d', 4)",
      "  assign('e', 5, 1)",
      "  innerfunc <- function() {",
      "    f <- 6",
      "    g <<- 7",
      "    assign('h', 8)",
      "  }",
      "}",
      sep = "\n"
    )),
    c("c", "d", "e", "g", "h")
  )

  # Variables assigned to should not appear in `uses` unless
  # used before assignement.
  chunk <- compile_code_chunk(paste(
    "b",
    "a <- 1",
    "a",
    "b <- 2",
    sep = "\n"
  ))
  expect_equal(
    chunk$assigns,
    c("a", "b")
  )
  expect_equal(
    chunk$uses,
    "b"
  )
})


test_that("compile_code_chunk: alters", {
  alters <- function(text) compile_code_chunk(text)$alters

  # The usual top level alters
  expect_equal(
    alters(paste(
      "var1[1] <- 1",
      "var2$prop = 2",
      "var3$prop <<- 3",
      "#' @alters var4, var5",
      "var6[[2]] <- 4",
      sep = "\n"
    )),
    c("var4", "var5", "var1", "var2", "var3", "var6")
  )

  # All alterations inside functions are ignored unless
  # they use the <<- operator
  expect_equal(
    alters(paste(
      "func <- function() {",
      "  a$b <- 1",
      "  c$d <<- 2",
      "  innerfunc <- function() {",
      "    e$f <- 6",
      "    g$h <<- 7",
      "    i[3] <<- 7",
      "  }",
      "}",
      sep = "\n"
    )),
    c("c", "g", "i")
  )

})
