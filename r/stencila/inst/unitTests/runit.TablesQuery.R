x = Column('x')
y = Column('y')

test.Query.constants <- function(){
  checkEquals('1',Constant(TRUE)$dql())
  checkEquals('42',Constant(42)$dql())
  checkEquals(3.14,as.numeric(Constant(3.14)$dql()))
  checkEquals("'banana'",Constant("banana")$dql())
}

test.Query.column <- function(){
  checkEquals('x',x$dql())
}

test.Query.unary <- function(){
  checkEquals('-x',(-x)$dql())
  checkEquals('+x',(+x)$dql())
  checkEquals('!x',(!x)$dql())
}

test.Query.binary <- function(){
  checkEquals('x*y',(x*y)$dql())
  checkEquals('x/y',(x/y)$dql())
  checkEquals('x+y',(x+y)$dql())
  checkEquals('x-y',(x-y)$dql())
  
  checkEquals('x==y',(x==y)$dql())
  checkEquals('x!=y',(x!=y)$dql())
  checkEquals('x<y',(x<y)$dql())
  checkEquals('x<=y',(x<=y)$dql())
  checkEquals('x>y',(x>y)$dql())
  checkEquals('x>=y',(x>=y)$dql())
  
  checkEquals('x and y',(x&y)$dql())
  checkEquals('x or y',(x|y)$dql())
  
  checkEquals("x in [1,2,3]",(x %in% c(1,2,3))$dql())
  checkEquals("x in ['a','b','c']",(x %in% c('a','b','c'))$dql())
  
  checkEquals('x*42',(x*42)$dql())
  checkEquals('42/y',(42/y)$dql())
  checkEquals("x=='banana'",(x=='banana')$dql())
  checkEquals("'banana'!=y",('banana'!=y)$dql())
}

test.Query.as <- function(){
  checkEquals('as("sum_of_x",sum(x))',As("sum_of_x",Call("sum",x))$dql())
}

test.Query.distinct.all <- function(){
  checkEquals('distinct()',Distinct()$dql())
  checkEquals('all()',All()$dql())
}

test.Query.where <- function(){
  checkEquals('where(x<2)',Where(x<2)$dql())
  checkEquals('where((x>2) and (x<2))',Where((x>2)&(x<2))$dql())
}

test.Query.by <- function(){
  checkEquals('by(x)',By(x)$dql())
}

test.Query.having <- function(){
  checkEquals('having(x>2)',Having(x>2)$dql())
}

test.Query.order <- function(){
  checkEquals('order(x)',Order(x)$dql())
}

test.Query.limit <- function(){
  checkEquals('limit(100)',Limit(100)$dql())
}

test.Query.offset <- function(){
  checkEquals('offset(100)',Offset(100)$dql())
}

test.Query.combiners <- function(){  
  checkEquals('top(by(x),sum(y),100)',Top(By(x),Aggregate('sum',y),100)$dql())
  checkEquals('top(by(x),sum(y),100)',Top(x,y,100)$dql())
}

test.Query.margins <- function(){  
  checkEquals('margin()',Margin()$dql())
  checkEquals('margin(by(x))',Margin(x)$dql())
}

test.Query.adjusters <- function(){  
  checkEquals('prop(y)',Proportion(y)$dql())
  checkEquals('prop(y,by(x))',Proportion(y,x)$dql())
  checkEquals('prop(sum(y),by(x))',Proportion(Aggregate('sum',y),By(x))$dql()) 
}

test.Query.combos <- function(){
  checkException(Query(),"a Query must be constructed with at least one argument")
  checkEquals(
    'x',
    Query(x)$dql()
  )
  checkEquals(
    'where(x<2)',
    Query(Where(x<2))$dql()
  )  
  checkEquals(
    'sum(x),where(x<2)',
    Query(Call('sum',x),Where(x<2))$dql()
  )  
  checkEquals(
    'as("sum_x",sum(x)),where(x<2)',
    Query(As("sum_x",Call('sum',x)),Where(x<2))$dql()
  )  
}

