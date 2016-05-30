#' A test runner that reads in `spreadsheet.xlsx` and
#' checks that when the test cell formulas are evaluated within the sheet
#' (using R compatibility functions) that the same cell values are obtained.

library(stencila)

# Import the xlsx into a sheet but don't execut it so it's
# values are still retained
sheet <- Sheet()
sheet$import('spreadsheet/spreadsheet.xlsx', execute=FALSE)

# Update the cells holding data so that their values are available
# when executing
sheet$update('J10:O50')

tests <- 0
fails <- 0
for (row in 10:500) {
	name <- sheet$cell(paste0('A',row))
	if (!is.null(name)) {
		for (col in c('B','C','D','E')) {
			test <- sheet$cell(paste0(col,row))
			if (!is.null(test)) {
				if (nchar(test$formula)>0) {
					tests <- tests + 1
					
					got <- sheet$evaluate(test$formula)
					
					# Convert spreadsheet calculated value to same
					# type as `got` for equality checking
					string <- test$value
					if (is.na(got)) {
						expected <- string
					} else if (mode(got)=='logical') {
						expected <- as.logical(as.numeric(string))
					} else if (mode(got)=='numeric') {
						# 'Round' to same number of significant digits for equality checking
						got <- signif(got,6)
						expected <- signif(as.numeric(string),6)
					} else {
						expected <- string
					}

					fail<- F
					if (is.na(got) | is.na(expected)) fail <- T
					else if (got != expected) fail <- T

					if(fail){
						cat('Fail ',col,row,' ',test$formula,': ',got,' != ',expected,' (',string,')','\n',sep='')
						fails <- fails + 1
					}
				}
			}
		}
	}
}
cat('Tests',tests,'\n')
cat('Fails',fails,'\n')
