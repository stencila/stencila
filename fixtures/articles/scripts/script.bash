# First paragraph, followed by some code

a=1

# Second paragraph, with a code expression that uses the above code `expr a * 2`{bash exec},
# followed by some more code, this time multiline,

b=2

function fn1 {}

# A multiline comment with a paragraph.
# 
# > And a quote
# > block
# 
# And a list,
# 
# 1. One
# 2. Two
# 3. Three

c=3

# Parameters can be included in Markdown, and will be encoded as script
# variables immediately after they are declared.
#
# Parameter d: &[d]{type=integer}

echo "a + b + c + d = $(expr $a + $b + $c + $d))"
