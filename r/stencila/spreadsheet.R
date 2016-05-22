#' Functions for compatability with spreadsheet formulas
#'
#' The function names and documentation were initially based on the list
#' of Microsoft Excel functions available at:
#'    https://support.office.com/en-us/article/Excel-functions-alphabetical-b3944572-255d-4efb-bb96-c6d90033e188
#'
#' For compatibility with other spreadsheet software additional functions may be added
#'
#' For some notes on mappings between spreadsheet functions and R see:
#'    http://www.burns-stat.com/spreadsheet-r-vector/
#'    http://www.rforexcelusers.com/r-functions-excel-formulas/

# Function to throw a not yet implemented error
nyi <- function() {
    stop(
        paste0(
            'Spreadsheet compatability function {', 
            sys.calls()[[sys.nframe()-1]][[1]],
            '} not yet implemented. See https://github.com/stencila/stencila/issues/172'
        ),
        call.=F
    )
}

# Function to assert a required argument
required <- function(name){
    stop(paste0('Parameter {',name,'} is required'),call.=F)
}

#' Returns the absolute value of a number
#'
#' @family Spreadsheet math and trigonometry
#' @export
ABS <- function(x){
    abs(x)
}


# Returns the accrued interest for a security that pays periodic interest
#
# @family Spreadsheet financial
# @export
# ACCRINT <- function(...){}


# Returns the accrued interest for a security that pays interest at maturity
#
# @family Spreadsheet financial
# @export
# ACCRINTM <- function(...){}


#' Returns the arccosine of a number
#'
#' @family Spreadsheet math and trigonometry
#' @export
ACOS <- function(x){
    acos(x)
}


#' Returns the inverse hyperbolic cosine of a number
#'
#' @family Spreadsheet math and trigonometry
#' @export
ACOSH <- function(x){
    acosh(x)
}


# Returns the arccotangent of a number
#
# @family Spreadsheet math and trigonometry
# @export
# ACOT <- function(...){}


# Returns the hyperbolic arccotangent of a number
#
# @family Spreadsheet math and trigonometry
# @export
# ACOTH <- function(...){}


# Returns an aggregate in a list or database
#
# @family Spreadsheet math and trigonometry
# @export
# AGGREGATE <- function(...){}


# Returns a reference as text to a single cell in a worksheet
#
# @family Spreadsheet lookup and reference
# @export
# ADDRESS <- function(...){}


# Returns the depreciation for each accounting period by using a depreciation coefficient
#
# @family Spreadsheet financial
# @export
# AMORDEGRC <- function(...){}


# Returns the depreciation for each accounting period
#
# @family Spreadsheet financial
# @export
# AMORLINC <- function(...){}


#' Returns TRUE if all of its arguments are TRUE
#'
#' @family Spreadsheet logical
#' @export
AND <- function(...){
    all(...)
}


# Converts a Roman number to Arabic, as a number
#
# @family Spreadsheet math and trigonometry
# @export
# ARABIC <- function(...){}


# Returns the number of areas in a reference
#
# @family Spreadsheet lookup and reference
# @export
# AREAS <- function(...){}


# Changes full-width (double-byte) English letters or katakana within a character string to half-width (single-byte) characters
#
# @family Spreadsheet text
# @export
# ASC <- function(...){}


#' Returns the arcsine of a number
#'
#' @family Spreadsheet math and trigonometry
#' @export
ASIN <- function(x){
    asin(x)
}


#' Returns the inverse hyperbolic sine of a number
#'
#' @family Spreadsheet math and trigonometry
#' @export
ASINH <- function(x){
    asinh(x)
}


#' Returns the arctangent of a number
#'
#' @family Spreadsheet math and trigonometry
#' @export
ATAN <- function(x){
    atan(x)
}


#' Returns the arctangent from x- and y-coordinates
#'
#' @family Spreadsheet math and trigonometry
#' @export
ATAN2 <- function(x,y){
    atan2(y,x)
}

#' Returns the inverse hyperbolic tangent of a number
#'
#' @family Spreadsheet math and trigonometry
#' @export
ATANH <- function(x){
    atanh(x)
}


#' Returns the average of the absolute deviations of data points from their mean
#'
#' @family Spreadsheet statistical
#' @export
AVEDEV <- function(...){
    x <- c(...)
    mean(abs(x-mean(x)))
}


#' Returns the average of its arguments
#'
#' @family Spreadsheet statistical
#' @export
AVERAGE <- function(...){
    mean(c(...))
}


# Returns the average of its arguments, including numbers, text, and logical values
#
# @family Spreadsheet statistical
# @export
# AVERAGEA <- function(...){}


#' Returns the average (arithmetic mean) of all the cells in a range that meet a given criteria
#'
#' @family Spreadsheet statistical
#'
#' @param range             Required. One or more cells to average, including numbers or names, arrays, 
#'                            or references that contain numbers.
#' @param criteria          Required. The criteria in the form of a number, expression, cell reference, 
#'                            or text that defines which cells are averaged. For example, criteria can be 
#'                            expressed as 32, "32", ">32", "apples", or B4.
#' @param average_range     Optional. The actual set of cells to average. If omitted, range is used.
#'
#' @examples 
#' AVERAGEIF(1:10, 5)
#' AVERAGEIF(1:10, "5")
#' AVERAGEIF(1:10, "<5")
#' AVERAGEIF(1:10, "<5", 10:1)
#' 
#' @export
AVERAGEIF <- function(range, criteria, average_range){
    if (missing(range)) required('range')
    if (missing(criteria)) required('criteria')
    if (length(criteria)!=1) stop('Argument {criteria} should have a length of 1')
    if (missing(average_range)) average_range = range
    
    # Determine the type of criteria
    if (is.character(criteria)){
      # Could be a character or an expression, find out...
      value <- tryCatch(eval(parse(text=criteria)), error=identity)
      if (inherits(value,'error')) {
        # Parsing failed so it (could be) an (incomplete) expression
        # which we need to make complete by pasteing it on to range
        # and then evaluate it
        selected <- tryCatch(eval(parse(text=paste0('range',criteria))), error=identity)
        if (inherits(selected,'error')) {
          stop(paste0('Error when evaluating the criteria {',criteria,'}'))
        }
      } else {
        # Parsing suceeded so just use the value e.g. "32", "apples"
        selected <- range==value
      }
    } else {
      # Criteria is probably a number e.g. 32 so compare to range values
      selected <- range==criteria
    }
    
    # Prevent cycling over selected if it's shorter than average range
    selected <- selected[1:length(average_range)]

    mean(average_range[selected])
}


# Returns the average (arithmetic mean) of all cells that meet multiple criteria.
#
# @family Spreadsheet statistical
# @export
# AVERAGEIFS <- function(...){}


# Converts a number to text, using the ß (baht) currency format
#
# @family Spreadsheet text
# @export
# BAHTTEXT <- function(...){}


# Converts a number into a text representation with the given radix (base)
#
# @family Spreadsheet math and trigonometry
# @export
# BASE <- function(...){}


#' Returns the modified Bessel function In(x)
#'
#' @family Spreadsheet engineering
#' @export
BESSELI <- function(x,n){
    besselI(x,n)
}


#' Returns the Bessel function Jn(x)
#'
#' @family Spreadsheet engineering
#' @export
BESSELJ <- function(x,n){
    besselJ(x,n)
}

#' Returns the modified Bessel function Kn(x)
#'
#' @family Spreadsheet engineering
#' @export
BESSELK <- function(x,n){
    besselK(x,n)
}

#' Returns the Bessel function Yn(x)
#'
#' @family Spreadsheet engineering
#' @export
BESSELY <- function(x,n){
    besselY(x,n)
}

#' Returns the beta cumulative distribution function NOTE:In Excel 2007, this is aStatisticalfunction.
#'
#' @family Spreadsheet compatibility
#' @export
BETADIST <- function(x, alpha, beta, lower=NULL, upper=NULL){
    BETA.DIST(x, alpha, beta, FALSE, lower, upper)
}


#' Returns the beta cumulative distribution function
#'
#' @family Spreadsheet statistical
#' @export
BETA.DIST <- function(x, alpha, beta, cumulative, lower=NULL, upper=NULL){
    if (!is.null(lower) | !(is.null(upper))) stop('Lower and upper not yet implemented')
    if (cumulative) pbeta(x, alpha, beta, lower.tail = TRUE) else dbeta(x, alpha, beta)
}


# Returns the inverse of the cumulative distribution function for a specified beta distribution NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# BETAINV <- function(...){}


# Returns the inverse of the cumulative distribution function for a specified beta distribution
#
# @family Spreadsheet statistical
# @export
# BETA.INV <- function(...){}


# Converts a binary number to decimal
#
# @family Spreadsheet engineering
# @export
# BIN2DEC <- function(...){}


# Converts a binary number to hexadecimal
#
# @family Spreadsheet engineering
# @export
# BIN2HEX <- function(...){}


# Converts a binary number to octal
#
# @family Spreadsheet engineering
# @export
# BIN2OCT <- function(...){}


# Returns the individual term binomial distribution probability NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# BINOMDIST <- function(...){}


# Returns the individual term binomial distribution probability
#
# @family Spreadsheet statistical
# @export
# BINOM.DIST <- function(...){}


# Returns the probability of a trial result using a binomial distribution
#
# @family Spreadsheet statistical
# @export
# BINOM.DIST.RANGE <- function(...){}


# Returns the smallest value for which the cumulative binomial distribution is less than or equal to a criterion value
#
# @family Spreadsheet statistical
# @export
# BINOM.INV <- function(...){}


# Returns a 'Bitwise And' of two numbers
#
# @family Spreadsheet engineering
# @export
# BITAND <- function(...){}


# Returns a value number shifted left by shift_amount bits
#
# @family Spreadsheet engineering
# @export
# BITLSHIFT <- function(...){}


# Returns a bitwise OR of 2 numbers
#
# @family Spreadsheet engineering
# @export
# BITOR <- function(...){}


# Returns a value number shifted right by shift_amount bits
#
# @family Spreadsheet engineering
# @export
# BITRSHIFT <- function(...){}


# Returns a bitwise 'Exclusive Or' of two numbers
#
# @family Spreadsheet engineering
# @export
# BITXOR <- function(...){}


# Calls a procedure in a dynamic link library or code resource
#
# @family Spreadsheet add-in and automation
# @export
# CALL <- function(...){}


# Rounds a number to the nearest integer or to the nearest multiple of significance
#
# @family Spreadsheet math and trigonometry
# @export
# CEILING <- function(...){}


# Rounds a number up, to the nearest integer or to the nearest multiple of significance
#
# @family Spreadsheet math and trigonometry
# @export
# CEILING.MATH <- function(...){}


# Rounds a number the nearest integer or to the nearest multiple of significance. Regardless of the sign of the number, the number is rounded up.
#
# @family Spreadsheet math and trigonometry
# @export
# CEILING.PRECISE <- function(...){}


# Returns information about the formatting, location, or contents of a cell NOTE: This function is not available in Excel Online.
#
# @family Spreadsheet information
# @export
# CELL <- function(...){}


# Returns the character specified by the code number
#
# @family Spreadsheet text
# @export
# CHAR <- function(...){}


# Returns the one-tailed probability of the chi-squared distribution NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# CHIDIST <- function(...){}


# Returns the inverse of the one-tailed probability of the chi-squared distribution NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# CHIINV <- function(...){}


# Returns the test for independence NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# CHITEST <- function(...){}


# Returns the cumulative beta probability density function
#
# @family Spreadsheet statistical
# @export
# CHISQ.DIST <- function(...){}


# Returns the one-tailed probability of the chi-squared distribution
#
# @family Spreadsheet statistical
# @export
# CHISQ.DIST.RT <- function(...){}


# Returns the cumulative beta probability density function
#
# @family Spreadsheet statistical
# @export
# CHISQ.INV <- function(...){}


# Returns the inverse of the one-tailed probability of the chi-squared distribution
#
# @family Spreadsheet statistical
# @export
# CHISQ.INV.RT <- function(...){}


# Returns the test for independence
#
# @family Spreadsheet statistical
# @export
# CHISQ.TEST <- function(...){}


# Chooses a value from a list of values
#
# @family Spreadsheet lookup and reference
# @export
# CHOOSE <- function(...){}


# Removes all nonprintable characters from text
#
# @family Spreadsheet text
# @export
# CLEAN <- function(...){}


# Returns a numeric code for the first character in a text string
#
# @family Spreadsheet text
# @export
# CODE <- function(...){}


# Returns the column number of a reference
#
# @family Spreadsheet lookup and reference
# @export
# COLUMN <- function(...){}


# Returns the number of columns in a reference
#
# @family Spreadsheet lookup and reference
# @export
# COLUMNS <- function(...){}


# Returns the number of combinations for a given number of objects
#
# @family Spreadsheet math and trigonometry
# @export
# COMBIN <- function(...){}


# Returns the number of combinations with repetitions for a given number of items
#
# @family Spreadsheet math and trigonometry
# @export
# COMBINA <- function(...){}


# Converts real and imaginary coefficients into a complex number
#
# @family Spreadsheet engineering
# @export
# COMPLEX <- function(...){}


# Combines the text from multiple ranges and/or strings, but it doesn't provide the delimiter or IgnoreEmpty arguments. NOTE:This function isn't available in Excel 2016 for Mac.
#
# @family Spreadsheet text
# @export
# CONCAT <- function(...){}


#' Joins several text items into one text item
#'
#' @family Spreadsheet text
#' @export
CONCATENATE <- function(...){
    paste0(...)
}


# Returns the confidence interval for a population mean NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# CONFIDENCE <- function(...){}


# Returns the confidence interval for a population mean
#
# @family Spreadsheet statistical
# @export
# CONFIDENCE.NORM <- function(...){}


# Returns the confidence interval for a population mean, using a Student's t distribution
#
# @family Spreadsheet statistical
# @export
# CONFIDENCE.T <- function(...){}


# Converts a number from one measurement system to another
#
# @family Spreadsheet engineering
# @export
# CONVERT <- function(...){}


# Returns the correlation coefficient between two data sets
#
# @family Spreadsheet statistical
# @export
# CORREL <- function(...){}


#' Returns the cosine of a number
#'
#' @family Spreadsheet math and trigonometry
#' @export
COS <- function(x){
    cos(x)
}


#' Returns the hyperbolic cosine of a number
#'
#' @family Spreadsheet math and trigonometry
#' @export
COSH <- function(x){
    cosh(x)
}

# Returns the hyperbolic cosine of a number
#
# @family Spreadsheet math and trigonometry
# @export
# COT <- function(...){}


# Returns the cotangent of an angle
#
# @family Spreadsheet math and trigonometry
# @export
# COTH <- function(...){}


# Counts how many numbers are in the list of arguments
#
# @family Spreadsheet statistical
# @export
# COUNT <- function(...){}


# Counts how many values are in the list of arguments
#
# @family Spreadsheet statistical
# @export
# COUNTA <- function(...){}


# Counts the number of blank cells within a range
#
# @family Spreadsheet statistical
# @export
# COUNTBLANK <- function(...){}


# Counts the number of cells within a range that meet the given criteria
#
# @family Spreadsheet statistical
# @export
# COUNTIF <- function(...){}


# Counts the number of cells within a range that meet multiple criteria
#
# @family Spreadsheet statistical
# @export
# COUNTIFS <- function(...){}


# Returns the number of days from the beginning of the coupon period to the settlement date
#
# @family Spreadsheet financial
# @export
# COUPDAYBS <- function(...){}


# Returns the number of days in the coupon period that contains the settlement date
#
# @family Spreadsheet financial
# @export
# COUPDAYS <- function(...){}


# Returns the number of days from the settlement date to the next coupon date
#
# @family Spreadsheet financial
# @export
# COUPDAYSNC <- function(...){}


# Returns the next coupon date after the settlement date
#
# @family Spreadsheet financial
# @export
# COUPNCD <- function(...){}


# Returns the number of coupons payable between the settlement date and maturity date
#
# @family Spreadsheet financial
# @export
# COUPNUM <- function(...){}


# Returns the previous coupon date before the settlement date
#
# @family Spreadsheet financial
# @export
# COUPPCD <- function(...){}


# Returns covariance, the average of the products of paired deviations NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# COVAR <- function(...){}


# Returns covariance, the average of the products of paired deviations
#
# @family Spreadsheet statistical
# @export
# COVARIANCE.P <- function(...){}


# Returns the sample covariance, the average of the products deviations for each data point pair in two data sets
#
# @family Spreadsheet statistical
# @export
# COVARIANCE.S <- function(...){}


# Returns the smallest value for which the cumulative binomial distribution is less than or equal to a criterion value NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# CRITBINOM <- function(...){}


# Returns the cosecant of an angle
#
# @family Spreadsheet math and trigonometry
# @export
# CSC <- function(...){}


# Returns the hyperbolic cosecant of an angle
#
# @family Spreadsheet math and trigonometry
# @export
# CSCH <- function(...){}


# Returns a key performance indicator (KPI) name, property, and measure, and displays the name and property in the cell. A KPI is a quantifiable measurement, such as monthly gross profit or quarterly employee turnover, used to monitor an organization's performance.
#
# @family Spreadsheet cube
# @export
# CUBEKPIMEMBER <- function(...){}


# Returns a member or tuple in a cube hierarchy. Use to validate that the member or tuple exists in the cube.
#
# @family Spreadsheet cube
# @export
# CUBEMEMBER <- function(...){}


# Returns the value of a member property in the cube. Use to validate that a member name exists within the cube and to return the specified property for this member.
#
# @family Spreadsheet cube
# @export
# CUBEMEMBERPROPERTY <- function(...){}


# Returns the nth, or ranked, member in a set. Use to return one or more elements in a set, such as the top sales performer or top 10 students.
#
# @family Spreadsheet cube
# @export
# CUBERANKEDMEMBER <- function(...){}


# Defines a calculated set of members or tuples by sending a set expression to the cube on the server, which creates the set, and then returns that set to Microsoft Office Excel.
#
# @family Spreadsheet cube
# @export
# CUBESET <- function(...){}


# Returns the number of items in a set.
#
# @family Spreadsheet cube
# @export
# CUBESETCOUNT <- function(...){}


# Returns an aggregated value from a cube.
#
# @family Spreadsheet cube
# @export
# CUBEVALUE <- function(...){}


# Returns the cumulative interest paid between two periods
#
# @family Spreadsheet financial
# @export
# CUMIPMT <- function(...){}


# Returns the cumulative principal paid on a loan between two periods
#
# @family Spreadsheet financial
# @export
# CUMPRINC <- function(...){}


# Returns the serial number of a particular date
#
# @family Spreadsheet date and time
# @export
# DATE <- function(...){}


# Calculates the number of days, months, or years between two dates. This function is useful in formulas where you need to calculate an age.
#
# @family Spreadsheet date and time
# @export
# DATEDIF <- function(...){}


# Converts a date in the form of text to a serial number
#
# @family Spreadsheet date and time
# @export
# DATEVALUE <- function(...){}


# Returns the average of selected database entries
#
# @family Spreadsheet database
# @export
# DAVERAGE <- function(...){}


# Converts a serial number to a day of the month
#
# @family Spreadsheet date and time
# @export
# DAY <- function(...){}


# Returns the number of days between two dates
#
# @family Spreadsheet date and time
# @export
# DAYS <- function(...){}


# Calculates the number of days between two dates based on a 360-day year
#
# @family Spreadsheet date and time
# @export
# DAYS360 <- function(...){}


# Returns the depreciation of an asset for a specified period by using the fixed-declining balance method
#
# @family Spreadsheet financial
# @export
# DB <- function(...){}


# Changes half-width (single-byte) English letters or katakana within a character string to full-width (double-byte) characters
#
# @family Spreadsheet text
# @export
# DBCS <- function(...){}


# Counts the cells that contain numbers in a database
#
# @family Spreadsheet database
# @export
# DCOUNT <- function(...){}


# Counts nonblank cells in a database
#
# @family Spreadsheet database
# @export
# DCOUNTA <- function(...){}


# Returns the depreciation of an asset for a specified period by using the double-declining balance method or some other method that you specify
#
# @family Spreadsheet financial
# @export
# DDB <- function(...){}


# Converts a decimal number to binary
#
# @family Spreadsheet engineering
# @export
# DEC2BIN <- function(...){}


# Converts a decimal number to hexadecimal
#
# @family Spreadsheet engineering
# @export
# DEC2HEX <- function(...){}


# Converts a decimal number to octal
#
# @family Spreadsheet engineering
# @export
# DEC2OCT <- function(...){}


# Converts a text representation of a number in a given base into a decimal number
#
# @family Spreadsheet math and trigonometry
# @export
# DECIMAL <- function(...){}


# Converts radians to degrees
#
# @family Spreadsheet math and trigonometry
# @export
# DEGREES <- function(...){}


# Tests whether two values are equal
#
# @family Spreadsheet engineering
# @export
# DELTA <- function(...){}


# Returns the sum of squares of deviations
#
# @family Spreadsheet statistical
# @export
# DEVSQ <- function(...){}


# Extracts from a database a single record that matches the specified criteria
#
# @family Spreadsheet database
# @export
# DGET <- function(...){}


# Returns the discount rate for a security
#
# @family Spreadsheet financial
# @export
# DISC <- function(...){}


# Returns the maximum value from selected database entries
#
# @family Spreadsheet database
# @export
# DMAX <- function(...){}


# Returns the minimum value from selected database entries
#
# @family Spreadsheet database
# @export
# DMIN <- function(...){}


# Converts a number to text, using the $ (dollar) currency format
#
# @family Spreadsheet text
# @export
# DOLLAR <- function(...){}


# Converts a dollar price, expressed as a fraction, into a dollar price, expressed as a decimal number
#
# @family Spreadsheet financial
# @export
# DOLLARDE <- function(...){}


# Converts a dollar price, expressed as a decimal number, into a dollar price, expressed as a fraction
#
# @family Spreadsheet financial
# @export
# DOLLARFR <- function(...){}


# Multiplies the values in a particular field of records that match the criteria in a database
#
# @family Spreadsheet database
# @export
# DPRODUCT <- function(...){}


# Estimates the standard deviation based on a sample of selected database entries
#
# @family Spreadsheet database
# @export
# DSTDEV <- function(...){}


# Calculates the standard deviation based on the entire population of selected database entries
#
# @family Spreadsheet database
# @export
# DSTDEVP <- function(...){}


# Adds the numbers in the field column of records in the database that match the criteria
#
# @family Spreadsheet database
# @export
# DSUM <- function(...){}


# Returns the annual duration of a security with periodic interest payments
#
# @family Spreadsheet financial
# @export
# DURATION <- function(...){}


# Estimates variance based on a sample from selected database entries
#
# @family Spreadsheet database
# @export
# DVAR <- function(...){}


# Calculates variance based on the entire population of selected database entries
#
# @family Spreadsheet database
# @export
# DVARP <- function(...){}


# Returns the serial number of the date that is the indicated number of months before or after the start date
#
# @family Spreadsheet date and time
# @export
# EDATE <- function(...){}


# Returns the effective annual interest rate
#
# @family Spreadsheet financial
# @export
# EFFECT <- function(...){}


# Returns a URL-encoded string NOTE: This function is not available in Excel Online.
#
# @family Spreadsheet web
# @export
# ENCODEURL <- function(...){}


# Returns the serial number of the last day of the month before or after a specified number of months
#
# @family Spreadsheet date and time
# @export
# EOMONTH <- function(...){}


# Returns the error function
#
# @family Spreadsheet engineering
# @export
# ERF <- function(...){}


# Returns the error function
#
# @family Spreadsheet engineering
# @export
# ERF.PRECISE <- function(...){}


# Returns the complementary error function
#
# @family Spreadsheet engineering
# @export
# ERFC <- function(...){}


# Returns the complementary ERF function integrated between x and infinity
#
# @family Spreadsheet engineering
# @export
# ERFC.PRECISE <- function(...){}


# Returns a number corresponding to an error type
#
# @family Spreadsheet information
# @export
# ERROR.TYPE <- function(...){}


# Converts a number to euros, converts a number from euros to a euro member currency, or converts a number from one euro member currency to another by using the euro as an intermediary (triangulation).
#
# @family Spreadsheet add-in and automation
# @export
# EUROCONVERT <- function(...){}


# Rounds a number up to the nearest even integer
#
# @family Spreadsheet math and trigonometry
# @export
# EVEN <- function(...){}


# Checks to see if two text values are identical
#
# @family Spreadsheet text
# @export
# EXACT <- function(...){}


#' Returnseraised to the power of a given number
#'
#' @family Spreadsheet math and trigonometry
#' @export
EXP <- function(x){
    exp(x)
}


# Returns the exponential distribution
#
# @family Spreadsheet statistical
# @export
# EXPON.DIST <- function(...){}


# Returns the exponential distribution NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# EXPONDIST <- function(...){}


# Returns the factorial of a number
#
# @family Spreadsheet math and trigonometry
# @export
# FACT <- function(...){}


# Returns the double factorial of a number
#
# @family Spreadsheet math and trigonometry
# @export
# FACTDOUBLE <- function(...){}


# Returns the F probability distribution
#
# @family Spreadsheet statistical
# @export
# F.DIST <- function(...){}


# Returns the F probability distribution NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# FDIST <- function(...){}


# Returns the F probability distribution
#
# @family Spreadsheet statistical
# @export
# F.DIST.RT <- function(...){}


# Returns specific data from the XML content by using the specified Xpath NOTE: This function is not available in Excel Online.
#
# @family Spreadsheet web
# @export
# FILTERXML <- function(...){}


# Finds one text value within another (case-sensitive)
#
# @family Spreadsheet text
# @export
# FIND <- function(...){}

# Finds one text value within another (case-sensitive)
#
# @family Spreadsheet text
# @export
# FINDB <- function(...){}


# Returns the inverse of the F probability distribution
#
# @family Spreadsheet statistical
# @export
# F.INV <- function(...){}


# Returns the inverse of the F probability distribution
#
# @family Spreadsheet statistical
# @export
# F.INV.RT <- function(...){}


# Returns the inverse of the F probability distribution
#
# @family Spreadsheet statistical
# @export
# FINV <- function(...){}


# Returns the Fisher transformation
#
# @family Spreadsheet statistical
# @export
# FISHER <- function(...){}


# Returns the inverse of the Fisher transformation
#
# @family Spreadsheet statistical
# @export
# FISHERINV <- function(...){}


# Formats a number as text with a fixed number of decimals
#
# @family Spreadsheet text
# @export
# FIXED <- function(...){}


#' Rounds a number down, toward zero
#'
#' @family Spreadsheet compatibility
#' @export
FLOOR <- function(x,significance){
    floor(x/significance)*significance
}


# Rounds a number down, to the nearest integer or to the nearest multiple of significance
#
# @family Spreadsheet math and trigonometry
# @export
# FLOOR.MATH <- function(...){}


# Rounds a number the nearest integer or to the nearest multiple of significance. Regardless of the sign of the number, the number is rounded up.
#
# @family Spreadsheet math and trigonometry
# @export
# FLOOR.PRECISE <- function(...){}


# Returns a value along a linear trend NOTE:In Excel 2016, this function is replaced withFORECAST.LINEARas part of the newForecasting functions, but it's still available for compatibility with earlier versions.
#
# @family Spreadsheet statistical
# @export
# FORECAST <- function(...){}


# Returns a future value based on existing (historical) values by using the AAA version of the Exponential Smoothing (ETS) algorithm NOTE:This function isn't available in Excel 2016 for Mac.
#
# @family Spreadsheet statistical
# @export
# FORECAST.ETS <- function(...){}


# Returns a confidence interval for the forecast value at the specified target date NOTE:This function isn't available in Excel 2016 for Mac.
#
# @family Spreadsheet statistical
# @export
# FORECAST.ETS.CONFINT <- function(...){}


# Returns the length of the repetitive pattern Excel detects for the specified time series NOTE:This function isn't available in Excel 2016 for Mac.
#
# @family Spreadsheet statistical
# @export
# FORECAST.ETS.SEASONALITY <- function(...){}


# Returns a statistical value as a result of time series forecasting NOTE:This function isn't available in Excel 2016 for Mac.
#
# @family Spreadsheet statistical
# @export
# FORECAST.ETS.STAT <- function(...){}


# Returns a future value based on existing values NOTE:This function isn't available in Excel 2016 for Mac.
#
# @family Spreadsheet statistical
# @export
# FORECAST.LINEAR <- function(...){}


# Returns the formula at the given reference as text
#
# @family Spreadsheet lookup and reference
# @export
# FORMULATEXT <- function(...){}


# Returns a frequency distribution as a vertical array
#
# @family Spreadsheet statistical
# @export
# FREQUENCY <- function(...){}


# Returns the result of an F-test
#
# @family Spreadsheet statistical
# @export
# F.TEST <- function(...){}


# Returns the result of an F-test NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# FTEST <- function(...){}


# Returns the future value of an investment
#
# @family Spreadsheet financial
# @export
# FV <- function(...){}


# Returns the future value of an initial principal after applying a series of compound interest rates
#
# @family Spreadsheet financial
# @export
# FVSCHEDULE <- function(...){}


# Returns the Gamma function value
#
# @family Spreadsheet statistical
# @export
# GAMMA <- function(...){}


# Returns the gamma distribution
#
# @family Spreadsheet statistical
# @export
# GAMMA.DIST <- function(...){}


# Returns the gamma distributionNOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# GAMMADIST <- function(...){}


# Returns the inverse of the gamma cumulative distribution
#
# @family Spreadsheet statistical
# @export
# GAMMA.INV <- function(...){}


# Returns the inverse of the gamma cumulative distribution NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# GAMMAINV <- function(...){}


# Returns the natural logarithm of the gamma function, Γ(x)
#
# @family Spreadsheet statistical
# @export
# GAMMALN <- function(...){}


# Returns the natural logarithm of the gamma function, Γ(x)
#
# @family Spreadsheet statistical
# @export
# GAMMALN.PRECISE <- function(...){}


# Returns 0.5 less than the standard normal cumulative distribution
#
# @family Spreadsheet statistical
# @export
# GAUSS <- function(...){}


# Returns the greatest common divisor
#
# @family Spreadsheet math and trigonometry
# @export
# GCD <- function(...){}


#' Returns the geometric mean
#'
#' @family Spreadsheet statistical
#' @export
GEOMEAN <- function(...){
    exp(mean(log(c(...))))
}


# Tests whether a number is greater than a threshold value
#
# @family Spreadsheet engineering
# @export
# GESTEP <- function(...){}


# Returns data stored in a PivotTable report
#
# @family Spreadsheet add-in and automation
# @export
# GETPIVOTDATA <- function(...){}


# Returns values along an exponential trend
#
# @family Spreadsheet statistical
# @export
# GROWTH <- function(...){}


#' Returns the harmonic mean
#'
#' @family Spreadsheet statistical
#' @export
HARMEAN <- function(...){
    1/mean(1/c(...))
}


# Converts a hexadecimal number to binary
#
# @family Spreadsheet engineering
# @export
# HEX2BIN <- function(...){}


# Converts a hexadecimal number to decimal
#
# @family Spreadsheet engineering
# @export
# HEX2DEC <- function(...){}


# Converts a hexadecimal number to octal
#
# @family Spreadsheet engineering
# @export
# HEX2OCT <- function(...){}


# Looks in the top row of an array and returns the value of the indicated cell
#
# @family Spreadsheet lookup and reference
# @export
# HLOOKUP <- function(...){}


# Converts a serial number to an hour
#
# @family Spreadsheet date and time
# @export
# HOUR <- function(...){}


# Creates a shortcut or jump that opens a document stored on a network server, an intranet, or the Internet
#
# @family Spreadsheet lookup and reference
# @export
# HYPERLINK <- function(...){}


# Returns the hypergeometric distribution
#
# @family Spreadsheet statistical
# @export
# HYPGEOM.DIST <- function(...){}


# Returns the hypergeometric distribution NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# HYPGEOMDIST <- function(...){}


# Specifies a logical test to perform
#
# @family Spreadsheet logical
# @export
# IF <- function(...){}


# Returns a value you specify if a formula evaluates to an error; otherwise, returns the result of the formula
#
# @family Spreadsheet logical
# @export
# IFERROR <- function(...){}


# Returns the value you specify if the expression resolves to #N/A, otherwise returns the result of the expression
#
# @family Spreadsheet logical
# @export
# IFNA <- function(...){}


# Checks whether one or more conditions are met and returns a value that corresponds to the first TRUE condition. NOTE:This function isn't available in Excel 2016 for Mac.
#
# @family Spreadsheet logical
# @export
# IFS <- function(...){}


# Returns the absolute value (modulus) of a complex number
#
# @family Spreadsheet engineering
# @export
# IMABS <- function(...){}


# Returns the imaginary coefficient of a complex number
#
# @family Spreadsheet engineering
# @export
# IMAGINARY <- function(...){}


# Returns the argument theta, an angle expressed in radians
#
# @family Spreadsheet engineering
# @export
# IMARGUMENT <- function(...){}


# Returns the complex conjugate of a complex number
#
# @family Spreadsheet engineering
# @export
# IMCONJUGATE <- function(...){}


# Returns the cosine of a complex number
#
# @family Spreadsheet engineering
# @export
# IMCOS <- function(...){}


# Returns the hyperbolic cosine of a complex number
#
# @family Spreadsheet engineering
# @export
# IMCOSH <- function(...){}


# Returns the cotangent of a complex number
#
# @family Spreadsheet engineering
# @export
# IMCOT <- function(...){}


# Returns the cosecant of a complex number
#
# @family Spreadsheet engineering
# @export
# IMCSC <- function(...){}


# Returns the hyperbolic cosecant of a complex number
#
# @family Spreadsheet engineering
# @export
# IMCSCH <- function(...){}


# Returns the quotient of two complex numbers
#
# @family Spreadsheet engineering
# @export
# IMDIV <- function(...){}


# Returns the exponential of a complex number
#
# @family Spreadsheet engineering
# @export
# IMEXP <- function(...){}


# Returns the natural logarithm of a complex number
#
# @family Spreadsheet engineering
# @export
# IMLN <- function(...){}


# Returns the base-10 logarithm of a complex number
#
# @family Spreadsheet engineering
# @export
# IMLOG10 <- function(...){}


# Returns the base-2 logarithm of a complex number
#
# @family Spreadsheet engineering
# @export
# IMLOG2 <- function(...){}


# Returns a complex number raised to an integer power
#
# @family Spreadsheet engineering
# @export
# IMPOWER <- function(...){}


# Returns the product of complex numbers
#
# @family Spreadsheet engineering
# @export
# IMPRODUCT <- function(...){}


# Returns the real coefficient of a complex number
#
# @family Spreadsheet engineering
# @export
# IMREAL <- function(...){}


# Returns the secant of a complex number
#
# @family Spreadsheet engineering
# @export
# IMSEC <- function(...){}


# Returns the hyperbolic secant of a complex number
#
# @family Spreadsheet engineering
# @export
# IMSECH <- function(...){}


# Returns the sine of a complex number
#
# @family Spreadsheet engineering
# @export
# IMSIN <- function(...){}


# Returns the hyperbolic sine of a complex number
#
# @family Spreadsheet engineering
# @export
# IMSINH <- function(...){}


# Returns the square root of a complex number
#
# @family Spreadsheet engineering
# @export
# IMSQRT <- function(...){}


# Returns the difference between two complex numbers
#
# @family Spreadsheet engineering
# @export
# IMSUB <- function(...){}


# Returns the sum of complex numbers
#
# @family Spreadsheet engineering
# @export
# IMSUM <- function(...){}


# Returns the tangent of a complex number
#
# @family Spreadsheet engineering
# @export
# IMTAN <- function(...){}


# Uses an index to choose a value from a reference or array
#
# @family Spreadsheet lookup and reference
# @export
# INDEX <- function(...){}


# Returns a reference indicated by a text value
#
# @family Spreadsheet lookup and reference
# @export
# INDIRECT <- function(...){}


# Returns information about the current operating environment NOTE: This function is not available in Excel Online.
#
# @family Spreadsheet information
# @export
# INFO <- function(...){}


# Rounds a number down to the nearest integer
#
# @family Spreadsheet math and trigonometry
# @export
# INT <- function(...){}


# Returns the intercept of the linear regression line
#
# @family Spreadsheet statistical
# @export
# INTERCEPT <- function(...){}


# Returns the interest rate for a fully invested security
#
# @family Spreadsheet financial
# @export
# INTRATE <- function(...){}


# Returns the interest payment for an investment for a given period
#
# @family Spreadsheet financial
# @export
# IPMT <- function(...){}


# Returns the internal rate of return for a series of cash flows
#
# @family Spreadsheet financial
# @export
# IRR <- function(...){}


# Returns TRUE if the value is blank
#
# @family Spreadsheet information
# @export
# ISBLANK <- function(...){}


# Returns TRUE if the value is any error value except #N/A
#
# @family Spreadsheet information
# @export
# ISERR <- function(...){}


# Returns TRUE if the value is any error value
#
# @family Spreadsheet information
# @export
# ISERROR <- function(...){}


# Returns TRUE if the number is even
#
# @family Spreadsheet information
# @export
# ISEVEN <- function(...){}


# Returns TRUE if there is a reference to a cell that contains a formula
#
# @family Spreadsheet information
# @export
# ISFORMULA <- function(...){}


# Returns TRUE if the value is a logical value
#
# @family Spreadsheet information
# @export
# ISLOGICAL <- function(...){}


# Returns TRUE if the value is the #N/A error value
#
# @family Spreadsheet information
# @export
# ISNA <- function(...){}


# Returns TRUE if the value is not text
#
# @family Spreadsheet information
# @export
# ISNONTEXT <- function(...){}


# Returns TRUE if the value is a number
#
# @family Spreadsheet information
# @export
# ISNUMBER <- function(...){}


# Returns TRUE if the number is odd
#
# @family Spreadsheet information
# @export
# ISODD <- function(...){}


# Returns TRUE if the value is a reference
#
# @family Spreadsheet information
# @export
# ISREF <- function(...){}


# Returns TRUE if the value is text
#
# @family Spreadsheet information
# @export
# ISTEXT <- function(...){}


# Returns a number that is rounded up to the nearest integer or to the nearest multiple of significance
#
# @family Spreadsheet math and trigonometry
# @export
# ISO.CEILING <- function(...){}


# Returns the number of the ISO week number of the year for a given date
#
# @family Spreadsheet date and time
# @export
# ISOWEEKNUM <- function(...){}


# Calculates the interest paid during a specific period of an investment
#
# @family Spreadsheet financial
# @export
# ISPMT <- function(...){}


# Changes half-width (single-byte) characters within a string to full-width (double-byte) characters
#
# @family Spreadsheet text
# @export
# JIS <- function(...){}


# Returns the kurtosis of a data set
#
# @family Spreadsheet statistical
# @export
# KURT <- function(...){}


# Returns the k-th largest value in a data set
#
# @family Spreadsheet statistical
# @export
# LARGE <- function(...){}


# Returns the least common multiple
#
# @family Spreadsheet math and trigonometry
# @export
# LCM <- function(...){}


# Returns the leftmost characters from a text value
#
# @family Spreadsheet text
# @export
# LEFT <- function(...){}

# Returns the leftmost characters from a text value
#
# @family Spreadsheet text
# @export
# LEFTB <- function(...){}


# Returns the number of characters in a text string
#
# @family Spreadsheet text
# @export
# LEN  <- function(...){}


# Returns the number of characters in a text string
#
# @family Spreadsheet text
# @export
# LENB <- function(...){}


# Returns the parameters of a linear trend
#
# @family Spreadsheet statistical
# @export
# LINEST <- function(...){}


#' Returns the natural logarithm of a number
#'
#' @family Spreadsheet math and trigonometry
#' @export
LN <- function(x){
    log(x)
}


#' Returns the logarithm of a number to a specified base
#'
#' @family Spreadsheet math and trigonometry
#' @export
LOG <- function(x, base){
    log(x, base = base)
}

#' Returns the base-10 logarithm of a number
#'
#' @family Spreadsheet math and trigonometry
#' @export
LOG10 <- function(x){
    log10(x)
}

# Returns the parameters of an exponential trend
#
# @family Spreadsheet statistical
# @export
# LOGEST <- function(...){}


# Returns the inverse of the lognormal cumulative distribution
#
# @family Spreadsheet compatibility
# @export
# LOGINV <- function(...){}


# Returns the cumulative lognormal distribution
#
# @family Spreadsheet statistical
# @export
# LOGNORM.DIST <- function(...){}


# Returns the cumulative lognormal distribution
#
# @family Spreadsheet compatibility
# @export
# LOGNORMDIST <- function(...){}


# Returns the inverse of the lognormal cumulative distribution
#
# @family Spreadsheet statistical
# @export
# LOGNORM.INV <- function(...){}


# Looks up values in a vector or array
#
# @family Spreadsheet lookup and reference
# @export
# LOOKUP <- function(...){}


# Converts text to lowercase
#
# @family Spreadsheet text
# @export
# LOWER <- function(...){}


# Looks up values in a reference or array
#
# @family Spreadsheet lookup and reference
# @export
# MATCH <- function(...){}


#' Returns the maximum value in a list of arguments
#'
#' @family Spreadsheet statistical
#' @export
MAX <- function(...){
    max(c(...))
}


# Returns the maximum value in a list of arguments, including numbers, text, and logical values
#
# @family Spreadsheet statistical
# @export
# MAXA <- function(...){}


# Returns the maximum value among cells specified by a given set of conditions or criteria NOTE:This function isn't available in Excel 2016 for Mac.
#
# @family Spreadsheet statistical
# @export
# MAXIFS <- function(...){}


# Returns the matrix determinant of an array
#
# @family Spreadsheet math and trigonometry
# @export
# MDETERM <- function(...){}


# Returns the Macauley modified duration for a security with an assumed par value of $100
#
# @family Spreadsheet financial
# @export
# MDURATION <- function(...){}


#' Returns the median of the given numbers
#'
#' @family Spreadsheet statistical
#' @export
MEDIAN <- function(...){
    median(c(...))
}


# Returns a specific number of characters from a text string starting at the position you specify
#
# @family Spreadsheet text
# @export
# MID <- function(...){}


# Returns a specific number of characters from a text string starting at the position you specify
#
# @family Spreadsheet text
# @export
# MIDB <- function(...){}


#' Returns the minimum value in a list of arguments
#'
#' @family Spreadsheet statistical
#' @export
MIN <- function(...){
    min(c(...))
}


# Returns the minimum value among cells specified by a given set of conditions or criteria. NOTE:This function isn't available in Excel 2016 for Mac.
#
# @family Spreadsheet statistical
# @export
# MINIFS <- function(...){}


# Returns the smallest value in a list of arguments, including numbers, text, and logical values
#
# @family Spreadsheet statistical
# @export
# MINA <- function(...){}


# Converts a serial number to a minute
#
# @family Spreadsheet date and time
# @export
# MINUTE <- function(...){}


# Returns the matrix inverse of an array
#
# @family Spreadsheet math and trigonometry
# @export
# MINVERSE <- function(...){}


# Returns the internal rate of return where positive and negative cash flows are financed at different rates
#
# @family Spreadsheet financial
# @export
# MIRR <- function(...){}


# Returns the matrix product of two arrays
#
# @family Spreadsheet math and trigonometry
# @export
# MMULT <- function(...){}


# Returns the remainder from division
#
# @family Spreadsheet math and trigonometry
# @export
# MOD <- function(...){}


#' Returns the most common value in a data set
#'
#' @family Spreadsheet compatibility
#' @export
MODE <- function(...){
    MODE.SNGL(...)
}


# Returns a vertical array of the most frequently occurring, or repetitive values in an array or range of data
#
# @family Spreadsheet statistical
# @export
# MODE.MULT <- function(...){}


#' Returns the most common value in a data set
#'
#' @family Spreadsheet statistical
#' @export
MODE.SNGL <- function(...){
    # Implementation thanks to @jmarhee
    # https://gist.github.com/jmarhee/8530768
    x <- c(...)
    ux <- unique(x)
    ux[which.max(tabulate(match(x, ux)))]
}


# Converts a serial number to a month
#
# @family Spreadsheet date and time
# @export
# MONTH <- function(...){}


# Returns a number rounded to the desired multiple
#
# @family Spreadsheet math and trigonometry
# @export
# MROUND <- function(...){}


# Returns the multinomial of a set of numbers
#
# @family Spreadsheet math and trigonometry
# @export
# MULTINOMIAL <- function(...){}


# Returns the unit matrix or the specified dimension
#
# @family Spreadsheet math and trigonometry
# @export
# MUNIT <- function(...){}


# Returns a value converted to a number
#
# @family Spreadsheet information
# @export
# N <- function(...){}


# Returns the negative binomial distribution
#
# @family Spreadsheet statistical
# @export
# NEGBINOM.DIST <- function(...){}


# Returns the negative binomial distribution NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# NEGBINOMDIST <- function(...){}


# Returns the number of whole workdays between two dates
#
# @family Spreadsheet date and time
# @export
# NETWORKDAYS <- function(...){}


# Returns the number of whole workdays between two dates using parameters to indicate which and how many days are weekend days
#
# @family Spreadsheet date and time
# @export
# NETWORKDAYS.INTL <- function(...){}


# Returns the annual nominal interest rate
#
# @family Spreadsheet financial
# @export
# NOMINAL <- function(...){}


# Returns the normal cumulative distribution
#
# @family Spreadsheet statistical
# @export
# NORM.DIST <- function(...){}


# Returns the normal cumulative distribution NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# NORMDIST <- function(...){}


# Returns the inverse of the normal cumulative distribution
#
# @family Spreadsheet statistical
# @export
# NORMINV <- function(...){}


# Returns the inverse of the normal cumulative distribution NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# NORM.INV <- function(...){}


# Returns the standard normal cumulative distribution
#
# @family Spreadsheet statistical
# @export
# NORM.S.DIST <- function(...){}


# Returns the standard normal cumulative distribution NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# NORMSDIST <- function(...){}


# Returns the inverse of the standard normal cumulative distribution
#
# @family Spreadsheet statistical
# @export
# NORM.S.INV <- function(...){}


# Returns the inverse of the standard normal cumulative distribution NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# NORMSINV <- function(...){}


# Reverses the logic of its argument
#
# @family Spreadsheet logical
# @export
# NOT <- function(...){}


# Returns the serial number of the current date and time
#
# @family Spreadsheet date and time
# @export
# NOW <- function(...){}


# Returns the number of periods for an investment
#
# @family Spreadsheet financial
# @export
# NPER <- function(...){}


# Returns the net present value of an investment based on a series of periodic cash flows and a discount rate
#
# @family Spreadsheet financial
# @export
# NPV <- function(...){}


# Converts text to number in a locale-independent manner
#
# @family Spreadsheet text
# @export
# NUMBERVALUE <- function(...){}


# Converts an octal number to binary
#
# @family Spreadsheet engineering
# @export
# OCT2BIN <- function(...){}


# Converts an octal number to decimal
#
# @family Spreadsheet engineering
# @export
# OCT2DEC <- function(...){}


# Converts an octal number to hexadecimal
#
# @family Spreadsheet engineering
# @export
# OCT2HEX <- function(...){}


# Rounds a number up to the nearest odd integer
#
# @family Spreadsheet math and trigonometry
# @export
# ODD <- function(...){}


# Returns the price per $100 face value of a security with an odd first period
#
# @family Spreadsheet financial
# @export
# ODDFPRICE <- function(...){}


# Returns the yield of a security with an odd first period
#
# @family Spreadsheet financial
# @export
# ODDFYIELD <- function(...){}


# Returns the price per $100 face value of a security with an odd last period
#
# @family Spreadsheet financial
# @export
# ODDLPRICE <- function(...){}


# Returns the yield of a security with an odd last period
#
# @family Spreadsheet financial
# @export
# ODDLYIELD <- function(...){}


# Returns a reference offset from a given reference
#
# @family Spreadsheet lookup and reference
# @export
# OFFSET <- function(...){}


# Returns TRUE if any argument is TRUE
#
# @family Spreadsheet logical
# @export
# OR <- function(...){}


# Returns the number of periods required by an investment to reach a specified value
#
# @family Spreadsheet financial
# @export
# PDURATION <- function(...){}


# Returns the Pearson product moment correlation coefficient
#
# @family Spreadsheet statistical
# @export
# PEARSON <- function(...){}


# Returns the k-th percentile of values in a range, where k is in the range 0..1, exclusive
#
# @family Spreadsheet statistical
# @export
# PERCENTILE.EXC <- function(...){}


# Returns the k-th percentile of values in a range
#
# @family Spreadsheet statistical
# @export
# PERCENTILE.INC <- function(...){}


# Returns the k-th percentile of values in a range NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# PERCENTILE <- function(...){}


# Returns the rank of a value in a data set as a percentage (0..1, exclusive) of the data set
#
# @family Spreadsheet statistical
# @export
# PERCENTRANK.EXC <- function(...){}


# Returns the percentage rank of a value in a data set
#
# @family Spreadsheet statistical
# @export
# PERCENTRANK.INC <- function(...){}


# Returns the percentage rank of a value in a data set NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# PERCENTRANK <- function(...){}


# Returns the number of permutations for a given number of objects
#
# @family Spreadsheet statistical
# @export
# PERMUT <- function(...){}


# Returns the number of permutations for a given number of objects (with repetitions) that can be selected from the total objects
#
# @family Spreadsheet statistical
# @export
# PERMUTATIONA <- function(...){}


# Returns the value of the density function for a standard normal distribution
#
# @family Spreadsheet statistical
# @export
# PHI <- function(...){}


# Extracts the phonetic (furigana) characters from a text string
#
# @family Spreadsheet text
# @export
# PHONETIC <- function(...){}


#' Returns the value of pi
#'
#' @family Spreadsheet math and trigonometry
#' @export
PI <- function(){
    pi
}


# Returns the periodic payment for an annuity
#
# @family Spreadsheet financial
# @export
# PMT <- function(...){}


# Returns the Poisson distribution
#
# @family Spreadsheet statistical
# @export
# POISSON.DIST <- function(...){}


# Returns the Poisson distribution NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# POISSON <- function(...){}


# Returns the result of a number raised to a power
#
# @family Spreadsheet math and trigonometry
# @export
# POWER <- function(...){}


# Returns the payment on the principal for an investment for a given period
#
# @family Spreadsheet financial
# @export
# PPMT <- function(...){}


# Returns the price per $100 face value of a security that pays periodic interest
#
# @family Spreadsheet financial
# @export
# PRICE <- function(...){}


# Returns the price per $100 face value of a discounted security
#
# @family Spreadsheet financial
# @export
# PRICEDISC <- function(...){}


# Returns the price per $100 face value of a security that pays interest at maturity
#
# @family Spreadsheet financial
# @export
# PRICEMAT <- function(...){}


# Returns the probability that values in a range are between two limits
#
# @family Spreadsheet statistical
# @export
# PROB <- function(...){}


# Multiplies its arguments
#
# @family Spreadsheet math and trigonometry
# @export
# PRODUCT <- function(...){}


# Capitalizes the first letter in each word of a text value
#
# @family Spreadsheet text
# @export
# PROPER <- function(...){}


# Returns the present value of an investment
#
# @family Spreadsheet financial
# @export
# PV <- function(...){}


# Returns the quartile of a data set NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# QUARTILE <- function(...){}


# Returns the quartile of the data set, based on percentile values from 0..1, exclusive
#
# @family Spreadsheet statistical
# @export
# QUARTILE.EXC <- function(...){}


# Returns the quartile of a data set
#
# @family Spreadsheet statistical
# @export
# QUARTILE.INC <- function(...){}


# Returns the integer portion of a division
#
# @family Spreadsheet math and trigonometry
# @export
# QUOTIENT <- function(...){}


# Converts degrees to radians
#
# @family Spreadsheet math and trigonometry
# @export
# RADIANS <- function(...){}


#' Returns a random number between 0 and 1
#'
#' @family Spreadsheet math and trigonometry
#' @export
RAND <- function(){
    runif(1)
}


#' Returns a random number between the numbers you specify
#'
#' @family Spreadsheet math and trigonometry
#' @export
RANDBETWEEN <- function(lower,upper){
    runif(1,lower,upper)
}

# Returns the rank of a number in a list of numbers
#
# @family Spreadsheet statistical
# @export
# RANK.AVG <- function(...){}


# Returns the rank of a number in a list of numbers
#
# @family Spreadsheet statistical
# @export
# RANK.EQ <- function(...){}


# Returns the rank of a number in a list of numbers NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# RANK <- function(...){}


# Returns the interest rate per period of an annuity
#
# @family Spreadsheet financial
# @export
# RATE <- function(...){}


# Returns the amount received at maturity for a fully invested security
#
# @family Spreadsheet financial
# @export
# RECEIVED <- function(...){}


# Returns the register ID of the specified dynamic link library (DLL) or code resource that has been previously registered
#
# @family Spreadsheet add-in and automation
# @export
# REGISTER.ID <- function(...){}


# Replaces characters within text
#
# @family Spreadsheet text
# @export
# REPLACE <- function(...){}


# Replaces characters within text
#
# @family Spreadsheet text
# @export
# REPLACEB <- function(...){}


# Repeats text a given number of times
#
# @family Spreadsheet text
# @export
# REPT <- function(...){}


# Returns the rightmost characters from a text value
#
# @family Spreadsheet text
# @export
# RIGHT <- function(...){}


# Returns the rightmost characters from a text value
#
# @family Spreadsheet text
# @export
# RIGHTB <- function(...){}


# Converts an arabic numeral to roman, as text
#
# @family Spreadsheet math and trigonometry
# @export
# ROMAN <- function(...){}


#' Rounds a number to a specified number of digits
#'
#' @family Spreadsheet math and trigonometry
#' @export
ROUND <- function(x, digits){
    round(x, digits)
}


# Rounds a number down, toward zero
#
# @family Spreadsheet math and trigonometry
# @export
# ROUNDDOWN <- function(...){}


# Rounds a number up, away from zero
#
# @family Spreadsheet math and trigonometry
# @export
# ROUNDUP <- function(...){}


# Returns the row number of a reference
#
# @family Spreadsheet lookup and reference
# @export
# ROW <- function(...){}


# Returns the number of rows in a reference
#
# @family Spreadsheet lookup and reference
# @export
# ROWS <- function(...){}


# Returns an equivalent interest rate for the growth of an investment
#
# @family Spreadsheet financial
# @export
# RRI <- function(...){}


# Returns the square of the Pearson product moment correlation coefficient
#
# @family Spreadsheet statistical
# @export
# RSQ <- function(...){}


# Retrieves real-time data from a program that supports COM automation
#
# @family Spreadsheet lookup and reference
# @export
# RTD <- function(...){}


# Finds one text value within another (not case-sensitive)
#
# @family Spreadsheet text
# @export
# SEARCH <- function(...){}


# Finds one text value within another (not case-sensitive)
#
# @family Spreadsheet text
# @export
# SEARCHB <- function(...){}


# Returns the secant of an angle
#
# @family Spreadsheet math and trigonometry
# @export
# SEC <- function(...){}


# Returns the hyperbolic secant of an angle
#
# @family Spreadsheet math and trigonometry
# @export
# SECH <- function(...){}


# Converts a serial number to a second
#
# @family Spreadsheet date and time
# @export
# SECOND <- function(...){}


# Returns the sum of a power series based on the formula
#
# @family Spreadsheet math and trigonometry
# @export
# SERIESSUM <- function(...){}


# Returns the sheet number of the referenced sheet
#
# @family Spreadsheet information
# @export
# SHEET <- function(...){}


# Returns the number of sheets in a reference
#
# @family Spreadsheet information
# @export
# SHEETS <- function(...){}


# Returns the sign of a number
#
# @family Spreadsheet math and trigonometry
# @export
# SIGN <- function(...){}


# Returns the sine of the given angle
#
# @family Spreadsheet math and trigonometry
# @export
# SIN <- function(...){}


# Returns the hyperbolic sine of a number
#
# @family Spreadsheet math and trigonometry
# @export
# SINH <- function(...){}


# Returns the skewness of a distribution
#
# @family Spreadsheet statistical
# @export
# SKEW <- function(...){}


# Returns the skewness of a distribution based on a population: a characterization of the degree of asymmetry of a distribution around its mean
#
# @family Spreadsheet statistical
# @export
# SKEW.P <- function(...){}


# Returns the straight-line depreciation of an asset for one period
#
# @family Spreadsheet financial
# @export
# SLN <- function(...){}


# Returns the slope of the linear regression line
#
# @family Spreadsheet statistical
# @export
# SLOPE <- function(...){}


# Returns the k-th smallest value in a data set
#
# @family Spreadsheet statistical
# @export
# SMALL <- function(...){}


# Connects with an external data source and runs a query from a worksheet, then returns the result as an array without the need for macro programming
#
# @family Spreadsheet add-in and automation
# @export
# SQL.REQUEST <- function(...){}


#' Returns a positive square root
#'
#' @family Spreadsheet math and trigonometry
#' @export
SQRT <- function(x){
    sqrt(x)
}


#' Returns the square root of (number * pi)
#'
#' @family Spreadsheet math and trigonometry
#' @export
SQRTPI <- function(x){
    sqrt(x*pi)
}


# Returns a normalized value
#
# @family Spreadsheet statistical
# @export
# STANDARDIZE <- function(...){}


# Estimates standard deviation based on a sample
#
# @family Spreadsheet compatibility
# @export
# STDEV <- function(...){}


# Calculates standard deviation based on the entire population
#
# @family Spreadsheet statistical
# @export
# STDEV.P <- function(...){}


# Estimates standard deviation based on a sample
#
# @family Spreadsheet statistical
# @export
# STDEV.S <- function(...){}


# Estimates standard deviation based on a sample, including numbers, text, and logical values
#
# @family Spreadsheet statistical
# @export
# STDEVA <- function(...){}


# Calculates standard deviation based on the entire population NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# STDEVP <- function(...){}


# Calculates standard deviation based on the entire population, including numbers, text, and logical values
#
# @family Spreadsheet statistical
# @export
# STDEVPA <- function(...){}


# Returns the standard error of the predicted y-value for each x in the regression
#
# @family Spreadsheet statistical
# @export
# STEYX <- function(...){}


# Substitutes new text for old text in a text string
#
# @family Spreadsheet text
# @export
# SUBSTITUTE <- function(...){}


# Returns a subtotal in a list or database
#
# @family Spreadsheet math and trigonometry
# @export
# SUBTOTAL <- function(...){}


#' Adds its arguments
#'
#' @family Spreadsheet math and trigonometry
#' @export
SUM <- function(...){
    sum(...)
}


# Adds the cells specified by a given criteria
#
# @family Spreadsheet math and trigonometry
# @export
# SUMIF <- function(...){}


# Adds the cells in a range that meet multiple criteria
#
# @family Spreadsheet math and trigonometry
# @export
# SUMIFS <- function(...){}


# Returns the sum of the products of corresponding array components
#
# @family Spreadsheet math and trigonometry
# @export
# SUMPRODUCT <- function(...){}


# Returns the sum of the squares of the arguments
#
# @family Spreadsheet math and trigonometry
# @export
# SUMSQ <- function(...){}


# Returns the sum of the difference of squares of corresponding values in two arrays
#
# @family Spreadsheet math and trigonometry
# @export
# SUMX2MY2 <- function(...){}


# Returns the sum of the sum of squares of corresponding values in two arrays
#
# @family Spreadsheet math and trigonometry
# @export
# SUMX2PY2 <- function(...){}


# Returns the sum of squares of differences of corresponding values in two arrays
#
# @family Spreadsheet math and trigonometry
# @export
# SUMXMY2 <- function(...){}


# Evaluates an expression against a list of values and returns the result corresponding to the first matching value. If there is no match, an optional default value may be returned. NOTE:This function isn't available in Excel 2016 for Mac.
#
# @family Spreadsheet logical
# @export
# SWITCH <- function(...){}


# Returns the sum-of-years' digits depreciation of an asset for a specified period
#
# @family Spreadsheet financial
# @export
# SYD <- function(...){}


#' Converts its arguments to text
#'
#' @family Spreadsheet text
#'
#' In Excel this function is called "T" but that name
#' clashes with Rs `T` i.e the alias for `TRUE`. In 
# `ExcelToRSheetGenerator` function calls to `T` are translated
# into `TEXT`
#
# @export
# TEXT <- function(...){}


# Returns the tangent of a number
#
# @family Spreadsheet math and trigonometry
# @export
# TAN <- function(...){}


# Returns the hyperbolic tangent of a number
#
# @family Spreadsheet math and trigonometry
# @export
# TANH <- function(...){}


# Returns the bond-equivalent yield for a Treasury bill
#
# @family Spreadsheet financial
# @export
# TBILLEQ <- function(...){}


# Returns the price per $100 face value for a Treasury bill
#
# @family Spreadsheet financial
# @export
# TBILLPRICE <- function(...){}


# Returns the yield for a Treasury bill
#
# @family Spreadsheet financial
# @export
# TBILLYIELD <- function(...){}


# Returns the Percentage Points (probability) for the Student t-distribution
#
# @family Spreadsheet statistical
# @export
# T.DIST <- function(...){}


# Returns the Percentage Points (probability) for the Student t-distribution
#
# @family Spreadsheet statistical
# @export
# T.DIST.2T <- function(...){}


# Returns the Student's t-distribution
#
# @family Spreadsheet statistical
# @export
# T.DIST.RT <- function(...){}


# Returns the Student's t-distribution
#
# @family Spreadsheet compatibility
# @export
# TDIST <- function(...){}


# Formats a number and converts it to text
#
# @family Spreadsheet text
# @export
# TEXT <- function(...){}


# Combines the text from multiple ranges and/or strings, and includes a delimiter you specify between each text value that will be combined. If the delimiter is an empty text string, this function will effectively concatenate the ranges. NOTE:This function isn't available in Excel 2016 for Mac.
#
# @family Spreadsheet text
# @export
# TEXTJOIN <- function(...){}


# Returns the serial number of a particular time
#
# @family Spreadsheet date and time
# @export
# TIME <- function(...){}


# Converts a time in the form of text to a serial number
#
# @family Spreadsheet date and time
# @export
# TIMEVALUE <- function(...){}


# Returns the t-value of the Student's t-distribution as a function of the probability and the degrees of freedom
#
# @family Spreadsheet statistical
# @export
# T.INV <- function(...){}


# Returns the inverse of the Student's t-distribution
#
# @family Spreadsheet statistical
# @export
# T.INV.2T <- function(...){}


# Returns the inverse of the Student's t-distribution
#
# @family Spreadsheet compatibility
# @export
# TINV <- function(...){}


# Returns the serial number of today's date
#
# @family Spreadsheet date and time
# @export
# TODAY <- function(...){}


# Returns the transpose of an array
#
# @family Spreadsheet lookup and reference
# @export
# TRANSPOSE <- function(...){}


# Returns values along a linear trend
#
# @family Spreadsheet statistical
# @export
# TREND <- function(...){}


# Removes spaces from text
#
# @family Spreadsheet text
# @export
# TRIM <- function(...){}


# Returns the mean of the interior of a data set
#
# @family Spreadsheet statistical
# @export
# TRIMMEAN <- function(...){}


# Truncates a number to an integer
#
# @family Spreadsheet math and trigonometry
# @export
# TRUNC <- function(...){}


# Returns the probability associated with a Student's t-test
#
# @family Spreadsheet statistical
# @export
# T.TEST <- function(...){}


# Returns the probability associated with a Student's t-test NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# TTEST <- function(...){}


# Returns a number indicating the data type of a value
#
# @family Spreadsheet information
# @export
# TYPE <- function(...){}


# Returns the Unicode character that is references by the given numeric value
#
# @family Spreadsheet text
# @export
# UNICHAR <- function(...){}


# Returns the number (code point) that corresponds to the first character of the text
#
# @family Spreadsheet text
# @export
# UNICODE <- function(...){}


# Converts text to uppercase
#
# @family Spreadsheet text
# @export
# UPPER <- function(...){}


# Converts a text argument to a number
#
# @family Spreadsheet text
# @export
# VALUE <- function(...){}


# Estimates variance based on a sample NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# VAR <- function(...){}


# Calculates variance based on the entire population
#
# @family Spreadsheet statistical
# @export
# VAR.P <- function(...){}


# Estimates variance based on a sample
#
# @family Spreadsheet statistical
# @export
# VAR.S <- function(...){}


# Estimates variance based on a sample, including numbers, text, and logical values
#
# @family Spreadsheet statistical
# @export
# VARA <- function(...){}


# Calculates variance based on the entire population NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# VARP <- function(...){}


# Calculates variance based on the entire population, including numbers, text, and logical values
#
# @family Spreadsheet statistical
# @export
# VARPA <- function(...){}


# Returns the depreciation of an asset for a specified or partial period by using a declining balance method
#
# @family Spreadsheet financial
# @export
# VDB <- function(...){}


# Looks in the first column of an array and moves across the row to return the value of a cell
#
# @family Spreadsheet lookup and reference
# @export
# VLOOKUP <- function(...){}


# Returns data from a web service. NOTE: This function is not available in Excel Online.
#
# @family Spreadsheet web
# @export
# WEBSERVICE <- function(...){}


# Converts a serial number to a day of the week
#
# @family Spreadsheet date and time
# @export
# WEEKDAY <- function(...){}


# Converts a serial number to a number representing where the week falls numerically with a year
#
# @family Spreadsheet date and time
# @export
# WEEKNUM <- function(...){}


# Calculates variance based on the entire population, including numbers, text, and logical values NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# WEIBULL <- function(...){}


# Returns the Weibull distribution
#
# @family Spreadsheet statistical
# @export
# WEIBULL.DIST <- function(...){}


# Returns the serial number of the date before or after a specified number of workdays
#
# @family Spreadsheet date and time
# @export
# WORKDAY <- function(...){}


# Returns the serial number of the date before or after a specified number of workdays using parameters to indicate which and how many days are weekend days
#
# @family Spreadsheet date and time
# @export
# WORKDAY.INTL <- function(...){}


# Returns the internal rate of return for a schedule of cash flows that is not necessarily periodic
#
# @family Spreadsheet financial
# @export
# XIRR <- function(...){}


# Returns the net present value for a schedule of cash flows that is not necessarily periodic
#
# @family Spreadsheet financial
# @export
# XNPV <- function(...){}


# Returns a logical exclusive OR of all arguments
#
# @family Spreadsheet logical
# @export
# XOR <- function(...){}


# Converts a serial number to a year
#
# @family Spreadsheet date and time
# @export
# YEAR <- function(...){}


# Returns the year fraction representing the number of whole days between start_date and end_date
#
# @family Spreadsheet date and time
# @export
# YEARFRAC <- function(...){}


# Returns the yield on a security that pays periodic interest
#
# @family Spreadsheet financial
# @export
# YIELD <- function(...){}


# Returns the annual yield for a discounted security; for example, a Treasury bill
#
# @family Spreadsheet financial
# @export
# YIELDDISC <- function(...){}


# Returns the annual yield of a security that pays interest at maturity
#
# @family Spreadsheet financial
# @export
# YIELDMAT <- function(...){}


# Returns the one-tailed probability-value of a z-test
#
# @family Spreadsheet statistical
# @export
# Z.TEST <- function(...){}


# Returns the one-tailed probability-value of a z-test NOTE:In Excel 2007, this is aStatisticalfunction.
#
# @family Spreadsheet compatibility
# @export
# ZTEST <- function(...){}
