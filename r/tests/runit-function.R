library(stencila)

f <- Function()
f$content(sum)
f$call(1:100)
print(f$json())
