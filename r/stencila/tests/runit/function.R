library(stencila)

f <- Function()
f$load('mean',format='name')
f$call(1:100)
print(f$json())
