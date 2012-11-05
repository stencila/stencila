test.Dataset.attributes <- function(){
  ds = Dataset()
  
  ds$execute("
      CREATE TABLE t1 (a INTEGER, b REAL, c TEXT);
      CREATE INDEX t1i1 ON t1(a);
      CREATE INDEX t1i2 ON t1(b);
     
      BEGIN TRANSACTION;
      INSERT INTO t1 VALUES(1,1.2,'alpha');
      INSERT INTO t1 VALUES(2,2.3,'beta');
      INSERT INTO t1 VALUES(3,3.4,'gamma');
      END TRANSACTION;
      
      CREATE TABLE t2 (a INTEGER);
      CREATE INDEX t2i1 ON t2(a);
  ")
  
  checkEquals(ds$tables(),c("t1","t2"))
  checkEquals(ds$indices(),c("t1i1","t1i2","t2i1"))
}



if(0){
  
  q = ds$cursor("SELECT * FROM t1")
  q$fetch()
  
  dt = ds$table("t1")
  
  print(dt)
  str(dt)
  
  dt$rows()
  dt$columns()
  
  dim(dt)
  nrow(dt)
  ncol(dt)
  
  dt$names()
  
  dt$type(0)
  dt$types()
  
  df = as.data.frame(dt)
  df
  is.factor(df$c)
  
  plot(dt)
  
  dt[1,]
  dt[1,1]
  dt[1:1,2:3]
  dt[1,'sales']
  dt[,'sales']
  
  dt[by(year),y=sum(sales),where(region=='E')]
  
  region_current = 'W'
  dt[by(year),y=sum(sales),where(region==region_current)]
  
}
