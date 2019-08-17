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

  # Using a full stop at end of tag prevents
  # augmentation by compiler
  expect_equal(
    imports(paste(
      "#' @imports pkg1, pkg2.",
      "library(pkg3)",
      sep = "\n"
    )),
    c("pkg1", "pkg2")
  )

  # Unquoted package names should not appear in `uses`
  # but other vars used in args should
  chunk <- compile_code_chunk(paste(
    "library(pkg1, pos = var1, quietly = TRUE)",
    "# @imports pkg2",
    "require(pkg3)",
    sep = "\n"
  ))
  expect_equal(
    chunk$imports,
    c("pkg2", "pkg1", "pkg3")
  )
  expect_equal(
    chunk$uses,
    "var1"
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

  expect_equal(
    alters(paste(
      "var1 <- list()",
      "var1$a <- 1", #Assigned locall so ignored
      "var2$a <- 1",
      "var3[0] <- 1",
      "var4[[0]] <- 1",
      sep = "\n"
    )),
    c("var2", "var3", "var4")
  )

  # Alterations in functions are ignored
  # except global assigns <<-
  expect_equal(
    alters(paste(
      "func1 <- function() {",
      "  var1[1] <- 1",
      "  innerfunc <- function() {",
      "    var2$a <- 2",
      "    var3$a <<- 2",
      "  }",
      "  var4[[1]] <<- 4",
      "}",
      sep = "\n"
    )),
    c("var3", "var4")
  )
})


test_that("compile_code_chunk: uses", {
  uses <- function(text) compile_code_chunk(text)$uses

  # The usual top level uses
  expect_equal(
    uses(paste(
      "var1",
      "var2 * 2",
      "sum(var3)",
      "#' @uses var4, var5",
      "plot(x~y, data)",
      "var6$a$b$c",
      "var7$method(var8)",
      sep = "\n"
    )),
    c("var4", "var5", "var1", "var2", "var3", "plot", "data", "var6", "var8")
  )

  # Uses of previous assigned or declared names are ignored
  expect_equal(
    uses(paste(
      "var1 <- 1",
      "var1",
      "var2",
      "var2 <- 2",
      "func1 <- function(){}",
      "func1()",
      "func2()",
      "func2 <- function(){}",
      sep = "\n"
    )),
    c("var2", "func2")
  )

  # Variables in NSE functions are ignored.
  expect_equal(
    uses(paste(
      "x ~ y",
      "subset(data1, foo > 1)",
      "filter(data2, bar < 1)",
      "dplyr::filter(data3, quax == 2)",
      sep = "\n"
    )),
    c("filter", "dplyr::filter")
  )
})

test_that("compile_code_chunk: reads", {
  reads <- function(text) compile_code_chunk(text)$reads

  expect_equal(
    reads(paste(
      "x <- read.csv('file1.csv')",
      "read.table(file = 'file2.csv')",
      "read.spss(sep = ',', file = 'file3.csv')",
      "#' @reads file4.csv",
      sep = "\n"
    )),
    c("file4.csv", "file1.csv", "file2.csv", "file3.csv")
  )
})

