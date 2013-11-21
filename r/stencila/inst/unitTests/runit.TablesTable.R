ds1 = Tableset()
ds1$execute("
  CREATE TABLE t1 (a INTEGER, b REAL, c TEXT);
  BEGIN TRANSACTION;
  INSERT INTO t1 VALUES(1,1.1,'a');
  INSERT INTO t1 VALUES(2,2.2,'b');
  INSERT INTO t1 VALUES(3,3.3,'c');
  END TRANSACTION;
")
dt1 <- ds1$table('t1')

test.Table.name <- function(){
  checkEquals("t1",dt1$name())
}

test.Table.dimensions <- function(){
  checkEquals(3,dt1$rows())
  checkEquals(3,dt1$columns())
  checkEquals(c(dt1$rows(),dt1$columns()),dt1$dimensions())
  checkEquals(dim(dt1),dt1$dimensions())
}  

test.Table.labels <- function(){
  checkEquals("a",dt1$label(0))
  checkEquals(c("a","b","c"),dt1$labels())
}

test.Table.types <- function(){
  checkEquals("Integer",dt1$type(0))
  checkEquals(c("Integer","Real","Text"),dt1$types())
}

test.Table.head <- function(){
  dt1$head()
  checkEquals(1.1,dt1$head(1)$value(0,1))
  checkEquals('a',dt1$head(1)$value(0,2))
}

test.Table.tail <- function(){
  dt1$tail()
  checkEquals(3.3,dt1$tail(1)$value(0,1))
}

test.Table.subscript.int.int <- function(){
  checkEquals(1,dt1[0,0])
  checkEquals(1.1,dt1[0,1])
  checkEquals('a',dt1[0,2])
  checkEquals(2.2,dt1[1,1])
}

test.Table.from_dataframe <- function(){
  df = data.frame(
    a = 1:5,
    b = c('a','b','c','d','e'),
    c = c(1.1,2.2,3.3,4.4,5.5)
  )
  dt = Table(df)
  
  show(dt)
  
  checkEquals(c(5,3),dt$dimensions())
  checkEquals(c('a','b','c'),dt$labels())
  checkEquals(c('Text','Text','Text'),dt$types())
  
  checkEquals(
      sum(df$a),
      dt[sum(a)]$value()
  )
}