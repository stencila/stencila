test.Dataquery.constants <- function(){
  checkEquals('1',Constant(TRUE)$dql())
  checkEquals('42',Constant(42)$dql())
  checkEquals(3.14,as.numeric(Constant(3.14)$dql()))
  checkEquals("'banana'",Constant("banana")$dql())
}

test.Dataquery.column <- function(){
  x = Column('x')
  checkEquals('x',x$dql())
}

test.Dataquery.unary <- function(){
  x = Column('x')
  checkEquals('-x',(-x)$dql())
  checkEquals('+x',(+x)$dql())
  checkEquals('!x',(!x)$dql())
}

test.Dataquery.binary <- function(){
  x = Column('x')
  y = Column('y')
  
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
  
  checkEquals('x*42',(x*42)$dql())
  checkEquals('42/y',(42/y)$dql())
  checkEquals("x=='banana'",(x=='banana')$dql())
  checkEquals("'banana'!=y",('banana'!=y)$dql())
}

test.Dataquery.where <- function(){
  x = Column('x')
  
  checkEquals('where(x<2)',Where(x<2)$dql())
  checkEquals('where((x>2) and (x<2))',Where((x>2)&(x<2))$dql())
}

test.Dataquery <- function(){
  x = Column('x')
  y = Column('y')
    
  check <- function(q,dql,sql){
    checkEquals(dql,q$dql())
    checkEquals(sql,q$sql())
  }

  check(
    Dataquery(),
    '<from>[]',
    'SELECT * FROM \"<from>\"'
  )
  
  check(
    Dataquery(Where(x<2)),
    '<from>[where(x<2)]',
    'SELECT * FROM \"<from>\" WHERE \"x\"<2'
  )  
  
  check(
    Dataquery(Call('sum',x),Where(x<2)),
    '<from>[sum(x),where(x<2)]',
    'SELECT sum(\"x\") FROM \"<from>\" WHERE \"x\"<2'
  )  
}

