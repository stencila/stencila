import unittest

from stencila import *


class DatatableBasics(unittest.TestCase):
    
    def nest_eql(self):
        
        a = Column('a')
        b = Column('b')
        
        self.assertEqual(a.dql(),'a')
        
        a_lt_42 = a<42
        self.assertTrue(isinstance(a_lt_42,LessThan))
        self.assertEqual(a_lt_42.dql(),'a<42')
        
        a_ge_pi = a>=3.14
        self.assertTrue(isinstance(a_ge_pi,GreaterEqual))
        self.assertEqual(a_ge_pi.dql(),'a>=3.1400001')
        
        a_ne_li = a!="lorem ipsum"
        self.assertTrue(isinstance(a_ne_li,NotEqual))
        self.assertEqual(a_ne_li.dql(),"a!='lorem ipsum'")
        
    def test_long_expressions(self):
        a = Column('a')
        b = Column('b')
        c = Column('c')
        
        #self.assertEqual(
        #    (a*5<10).dql(),
        #    "a*5<10"
        #)
        
        #self.assertEqual(
        #    (b/4+1).dql(),
        #    "b/4+1"
        #)
        p0 = a+3
        print p0.dql()
        print '------------'
        
        p1 = p0-c*2
        print p0.dql()
        print p1.dql()
        print '------------'
        
        p2 = b/58
        print p0.dql()
        print p1.dql()
        print p2.dql()
        print '------------'
        
        p3 = p1>=p2
        print p0.dql()
        print p1.dql()
        print p2.dql()
        print p3.dql()
        print '------------'
        
        if 0:
            self.assertEqual(
                ((a*5<10) | ((b/4+1>6) & (c!='foo'))).dql(),
                "a*5<10 | b/4+1>6 & c!='foo'"
            )


def EnterExitWith():
    '''
    Test enter, exit methods and context manager functionality of Datatables
    
    This is not run using the unittest module as that module
    seems to break the enter() method.
    '''

    #Create a Databale with a single column
    dt = Datatable()
    dt.add("c1",Text)

    #Enter the Datatable and check that column has been imported into the local namespace
    dt.enter()
    assert locals().has_key('c1')
    assert isinstance(c1,Column)
    
    #Exit the Datatable and check that the colun is NOT in the local namespace
    dt.exit()
    assert not locals().has_key('c1')
    try:
        print c1
    except:
        pass
    else:
        raise Exception

    #Check that enter() and exit() methods work with the "with" keyword
    with dt:
         assert locals().has_key('c1')
         assert isinstance(c1,Column)
    assert not locals().has_key('c1')
            
if __name__ == '__main__':
   unittest.main()
   EnterExitWith()
    
