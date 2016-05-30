# Runs the unit tests in the unitTests directory using the RUnit
# package.
library(stencila)
library(RUnit)

# Create a test suite from all runit.*.R files in the runit directory
suite <- defineTestSuite(
  "tests",
  dirs = file.path("runit"),
  testFileRegexp = '^.+\\.R'
)
# Run the test suite
result <- runTestSuite(suite)
# Report test results in junit format
printTextProtocol(result,fileName="RUnit.txt",showDetails=TRUE)
