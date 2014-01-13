# Runs the unit tests in the unitTests directory using the RUnit
# package.

library(RUnit)

# Create a test suite from all runit.*.R files in the unitTests directory
suite <- defineTestSuite(
  "tests",
  dirs = file.path("."),
  testFileRegexp = '^runit-.+\\.R'
)
# Run the test suite
result <- runTestSuite(suite)
# Report test results in junit format
printTextProtocol(result,fileName="RUnit.txt",showDetails=TRUE)
