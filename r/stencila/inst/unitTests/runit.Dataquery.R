x = Column('x')
y = Column('y')

test.Dataquery.constants <- function(){
  checkEquals('1',Constant(TRUE)$dql())
  checkEquals('42',Constant(42)$dql())
  checkEquals(3.14,as.numeric(Constant(3.14)$dql()))
  checkEquals("'banana'",Constant("banana")$dql())
}

test.Dataquery.column <- function(){
  checkEquals('x',x$dql())
}

test.Dataquery.unary <- function(){
  checkEquals('-x',(-x)$dql())
  checkEquals('+x',(+x)$dql())
  checkEquals('!x',(!x)$dql())
}

test.Dataquery.binary <- function(){
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

test.Dataquery.as <- function(){
  checkEquals('as("sum_of_x",sum(x))',As("sum_of_x",Call("sum",x))$dql())
}

test.Dataquery.distinct.all <- function(){
  checkEquals('distinct()',Distinct()$dql())
  checkEquals('all()',All()$dql())
}

test.Dataquery.where <- function(){
  checkEquals('where(x<2)',Where(x<2)$dql())
  checkEquals('where((x>2) and (x<2))',Where((x>2)&(x<2))$dql())
}

test.Dataquery.by <- function(){
  checkEquals('by(x)',By(x)$dql())
}

test.Dataquery.having <- function(){
  checkEquals('having(x>2)',Having(x>2)$dql())
}

test.Dataquery.order <- function(){
  checkEquals('order(x)',Order(x)$dql())
}

test.Dataquery.limit <- function(){
  checkEquals('limit(100)',Limit(100)$dql())
}

test.Dataquery.offset <- function(){
  checkEquals('offset(100)',Offset(100)$dql())
}

test.Dataquery.combiners <- function(){  
  checkEquals('top(by(x),sum(y),100)',Top(By(x),Aggregate('sum',y),100)$dql())
  checkEquals('top(by(x),sum(y),100)',Top(x,y,100)$dql())
}

test.Dataquery.margins <- function(){  
  checkEquals('margin()',Margin()$dql())
  checkEquals('margin(by(x))',Margin(x)$dql())
}

test.Dataquery.adjusters <- function(){  
  checkEquals('prop(y)',Proportion(y)$dql())
  checkEquals('prop(y,by(x))',Proportion(y,x)$dql())
  checkEquals('prop(sum(y),by(x))',Proportion(Aggregate('sum',y),By(x))$dql()) 
}

test.Dataquery.combos <- function(){
  checkException(Dataquery(),"a Dataquery must be constructed with at least one argument")
  checkEquals(
    'x',
    Dataquery(x)$dql()
  )
  checkEquals(
    'where(x<2)',
    Dataquery(Where(x<2))$dql()
  )  
  checkEquals(
    'sum(x),where(x<2)',
    Dataquery(Call('sum',x),Where(x<2))$dql()
  )  
  checkEquals(
    'as("sum_x",sum(x)),where(x<2)',
    Dataquery(As("sum_x",Call('sum',x)),Where(x<2))$dql()
  )  
}

