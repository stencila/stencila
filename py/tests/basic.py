import unittest
import os

import stencila

dataset_sql = '''
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
'''

class DatasetBasics(unittest.TestCase):
    
    def setUp(self):
        self.ds = stencila.Dataset()
        self.ds.execute(dataset_sql)

    def test_tables(self):
        self.assertEqual(self.ds.tables(),['t1','t2'])
        
    def test_indices(self):
        self.assertEqual(self.ds.indices(),['t1i1','t1i2','t2i1'])
        self.assertEqual(self.ds.indices('t2'),['t2i1'])
        
    def test_fetch(self):
        self.assertEqual(self.ds.fetch("SELECT * FROM t1;"),[
            [1, 1.2, 'alpha'],
            [2, 2.3, 'beta'],
            [3, 3.4, 'gamma']
        ])
        
    def test_value(self):
        self.assertEqual(self.ds.value("SELECT sum(a) FROM t1;"),6)
        self.assertEqual(self.ds.value("SELECT sum(b) FROM t1;"),6.9)
        self.assertEqual(self.ds.value("SELECT c FROM t1;"),'alpha')
        
    def test_column(self):
        self.assertEqual(self.ds.column("SELECT a FROM t1;"),[1,2,3])
        
    def test_row(self):
        row = self.ds.row("SELECT max(a),max(b),max(c) FROM t1;")
        self.assertEqual(row,[3,3.4,'gamma'])


class DatasetPermanant(unittest.TestCase):
    
    def setUp(self):
        self.ds = stencila.Dataset('test.sds')
        
    def tearDown(self):
        os.remove('test.sds')
        
        
class DatatableBasics(unittest.TestCase):

    def setUp(self):
        self.ds = stencila.Dataset()
        self.ds.execute(dataset_sql)
        self.dt = self.ds.table('t1')
        
    def test_rows(self):
        self.assertEqual(self.dt.rows(),3)
        
    def test_columns(self):
        self.assertEqual(self.dt.columns(),3)
        
    def test_names(self):
        self.assertEqual(self.dt.names(),['a','b','c'])
        
    def test_indices(self):
        self.assertEqual(self.dt.indices(),['t1i1','t1i2'])
    

if __name__ == '__main__':
    unittest.main()
    


