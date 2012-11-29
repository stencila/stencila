test.Datatable <- function(){
  ds = Dataset()
  ds$execute("
      CREATE TABLE t1 (a INTEGER, b REAL, c TEXT);
  ")
  dt = ds$table('t1')
  
  print(dt[a<1])
  print(dt[2,"hello",sum(a),where(a>2),by(c)])
}
