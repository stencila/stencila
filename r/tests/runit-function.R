library(stencila)

f <- Function()
f$load(sum)
f$call(1:100)
print(f$json())
