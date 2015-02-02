#' @include extension.R
NULL

#' The Component class
#'
#' @name Component
#' @export
Component <- function() {
    new('Component')
}
setRefClass(
	'Component',
    contains = 'Extension',
	fields = list(
		path = function(value) get_set_(.self,'Component_path_get','Component_path_set',value),
		address = function(value) get_(.self,'Component_address_get'),

		origin = function(value) get_(.self,'Component_origin_get'),
		commits = function(value) get_(.self,'Component_commits_get')
	),
	methods = list(
		show = function(){
		    cat(class(.self)[1],'@',address,'\n',sep='')
		},
		commit = function(message=""){
			method_(.self,'Component_commit',toString(message))
		},

		test = function(task='run'){
			runners <- list(
				runit = list(
					setup = function(){
						dir <- file.path(.self$path,'tests/r/runit')
						if(!file.exists(dir)) dir.create(dir,recursive=TRUE)
						fileName <- file.path(dir,'tests.R')
						if(!file.exists(fileName)){
							fileConn <- file(fileName)
							writeLines(c(
								"require(stencila)",
								"require(RUnit)",
								"",
								"# Write a bunch of `text.*` functions using the `check*` family",
								"# of assertions. Something like this...",
								"#",
								"#   self <- Component('.')",
								"#",
								"#   test.fourty_two <- function(){",
								"#       checkEquals(self$method(),42)",
								"#   }",
								""
							), fileConn)
							close(fileConn)
						}
					},
					run = function(){
						require(RUnit)
						dir <- file.path(.self$path,'tests/r/runit')
						# Create a test suite from all test*.R files in the tests/r/runit directory
						suite <- defineTestSuite(
							"tests",
							dirs = dir,
							testFileRegexp = '^test.+\\.R'
						)
						# Run the test suite
						result <- runTestSuite(suite)
						# Report test results in junit format
						printTextProtocol(
							result,
							fileName = file.path(dir,'results.txt'),
							showDetails = TRUE
						)
					}
				),
				testthat = list(
					setup = function(){
						dir <- file.path(.self$path,'tests/r/testthat')
						if(!file.exists(dir)) dir.create(dir,recursive=TRUE)
						fileName <- file.path(dir,'tests.R')
						if(!file.exists(fileName)){
							fileConn <- file(fileName)
							writeLines(c(
								"require(stencila)",
								"require(testthat)",
								"",
								"# Write a bunch of `test_that` functions using the `expect_*` family",
								"# of assertions. Something like this...",
								"#",
								"#   self <- Component('.')",
								"#",
								"#   test_that('something works',function(){",
								"#       expect_equal(self$method(),42)",
								"#   })",
								""
							), fileConn)
							close(fileConn)
						}
					},
					run = function(){
						# Run tests
						require(testthat)
						dir <- file.path(.self$path,'tests/r/testthat')
						results <- test_dir(dir)
						# Write results
						write.table(results,file.path(dir,'results.txt'),row.names=F,col.names=T,quote=F)
					}
				)
			)
			for(runner in runners){
				runner[[task]]()
			}
		}
	)
)
