source('stencil.R')

a = 'A1'
c2 = data.frame(a='A2',stringsAsFactors=F)
c3 = list(
  a='A3',
  b=data.frame(
      a='A4',
      stringsAsFactors=F
  )
)
co = Context()

co$enter('data.frame(x=1)')
co$stack
co$pop()
co$stack

co$get('a') #A1

co$enter('c2')
co$get('a') #A2

co$enter('c3')
co$get('a') #A3

co$enter('b')
co$get('a') #A4

co$exit()
co$get('a') #A3

co$exit()
co$get('a') #A2

co$exit()
co$get('a') #A1

####################

co$enter()
co$set('x','2')
co$get('x')

co$enter()
co$set('y','3')
co$get('x*y')

co$exit()

co$exit()

#####################
items = c('a','b','c')
it = iterate(items)
it$more()
it$step()
it$step()
it$step()
it$more()

co$begin('item','items')
co$step()

co$get('item') #a

co$step() 
co$get('item') #b

co$step() 
co$get('item') #c

co$step()
co$get('item') #should raise error




