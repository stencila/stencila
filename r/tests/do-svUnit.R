# Runs the unit tests in the unitTests directory using the svUnit
# package. svUnit provides for JUnit style XMl reporting of test results
# which can be integrated with the Stencila Continuous Integration Server (ci.stenci.la)
# Requires both svUnit and XML packages:
#     install.packages(c('svUnit','XML'))

library(svUnit)

# Create a test suite from all runit.*.R files in the unitTests directory
suite <- svSuite("package:stencila")
# Clear the test log 
clearLog()
# Run the test suite
runTest(suite)
# Report test results in junit format
protocol_junit(Log(),file="svUnit.xml")
# Count the number of failiures and errors
fails <- sum(stats(Log())$kind %in% c('**FAILS**','**ERROR**'))
