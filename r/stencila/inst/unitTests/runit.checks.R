test.checks <- function(){
  # This test is simply a reference providing examples of the various
  # check functions that are available
  checkEquals(6, factorial(3))
  checkEquals(c(1,2,3), c(1,2,3))
  checkEqualsNumeric(6, factorial(3))
  checkIdentical(6, factorial(3))
  checkTrue(2 + 2 == 4, 'Arithmetic works')
  checkException(log('a'), 'Unable to take the log() of a string')
}
