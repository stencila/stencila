library(testthat)
library(stencila)

sheet <- Sheet()
sheet$import('spreadsheet.xlsx')

for (row in 1:500) {
	name <- sheet$cell(paste0('A',row))
	if (!is.null(name)) {
		for (col in c('B','C','D','E')) {
			test <- sheet$cell(paste0(col,row))
			if (!is.null(test)) {
				if (nchar(test$formula)>0) {
					test_that(paste0(col,row,": ",test$formula),{
						got <- eval(parse(text=test$formula))
						expected <- test$value
						if (mode(got)=='logical') {
							expected <- as.logical(expected)
						} else if (mode(got)=='numeric') {
							expected <- as.numeric(expected)
						}
						expect_equal(got, expected)
					})
				}
			}
		}
	}
}
