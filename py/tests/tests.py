import unittest

import stencila
from stencila import *

class ExceptionTests(unittest.TestCase):

    def test_translation(self):
        try:
            stencila.extension.exception_test()
        except:
            pass
        else:
            raise


class ComponentTests(unittest.TestCase):

    def setUp(self):
        self.component = stencila.Package()

    def tearDown(self):
        self.component.destroy()

    def test_title(self):
        title = "A really useful component"
        self.assertEqual(self.component.title(title).title(),title)

    def test_description(self):
        desc = "A not so useful description of a really useful component"
        self.assertEqual(self.component.description(desc).description(),desc)

    def test_keywords(self):
        self.assertEqual(self.component.keywords(),[])

    def test_authors(self):
        self.assertEqual(self.component.authors(),[])

    # self.assertRegex not in all versions of Python
    #def test_path(self):
    #    self.assertRegex(self.component.path("").path(),"~")

    def test_create(self):
        self.component.create("README.txt")

    def test_destroy(self):
        self.component.delete("README.txt")

    def test_read(self):
        self.component.read()

    def test_write(self):
        self.component.write()

    def test_commit(self):
        self.component.commit("Just a test commit")


class PackageTests(unittest.TestCase):
    
    pass


class NamespaceTests(unittest.TestCase):
    '''
    Tests for the Namespace class
    '''

    def test(self):
        '''
        A simple test of name resolution
        '''

        a = 'a0'

        # Create a namespace that will have read-only access
        # to the current frame's locals
        ns1 = stencila.Namespace({'a':a})
        ns1['b'] = 'b1'
        ns1['c'] = 'c1'

        ## Create a child namespace
        ns2 = stencila.Namespace(ns1)
        ns2['c'] = 'c2'
        ns2['d'] = 'd2'

        # Check that the correct variables are obtained
        self.assertEqual(ns2['a'],'a0')
        self.assertEqual(ns2['b'],'b1')
        self.assertEqual(ns2['c'],'c2')

        # The local variable 'a' is read-only within the namespace
        ns2['a'] = 'a0-is-not-changed'
        self.assertEqual(a,'a0')
        self.assertNotEqual(a,'a0-is-not-changed')
        self.assertEqual(ns1['a'],'a0')

        # Check that a KeyError is thrown if no such name
        self.assertRaises(KeyError,ns2.__getitem__,'foo')
        self.assertRaises(KeyError,ns1.__getitem__,'d')


class ContextTests(unittest.TestCase):
    '''
    Tests for the Context class
    '''

    def test_execute(self):
        c = Context()
        c.execute('''
a = 1.2
b = 3.4
c = a+b
        ''')
        self.assertEqual(c.get('c'),4.6)

    def test_assign(self):
        c = Context()
        c.assign('x','4*3')
        self.assertEqual(c.get('x'),12)

    def test_input(self):
        c = Context()
        c.input('x','number','42')
        self.assertEqual(c.get('x'),42)

    def test_write(self):
        c = Context()
        self.assertEqual(c.set('x',42).write('x'),'42')
        self.assertEqual(c.set('x',[1,2,3]).write('x'),'[1, 2, 3]')
        self.assertEqual(c.set('x','foo').write('x'),'foo')

    def test_paint(self):
        pass

    def test_test(self):
        c = Context()
        c.set('x',42)
        self.assertTrue(c.test('x==42'))
        self.assertTrue(c.test('x==6*7'))
        self.assertTrue(c.test('x!=43'))

    def test_mark_match_unmark(self):
        c = Context()
        c.set('x',42)

        c.mark('x')
        self.assertTrue(c.match('42'))
        self.assertTrue(c.match('6*7'))
        self.assertTrue(not c.match('43'))
        c.unmark()

        self.assertRaises(ContextError,c.match,'foo')

    def test_begin_next(self):
        c = Context()
        self.assertTrue(c.begin('fruit','["apple","pear","kiwifruit"]'))
        self.assertEqual(c.get('fruit'),'apple')
        self.assertTrue(c.next())
        self.assertEqual(c.get('fruit'),'pear')
        self.assertTrue(c.next())
        self.assertEqual(c.get('fruit'),'kiwifruit')
        self.assertFalse(c.next())

    def test_enter_exit(self):
        c = Context()
        c.set('x',{'a':42,'b':'foo'})
        c.enter('x')
        self.assertEqual(c.get('a'),42)
        self.assertEqual(c.get('b'),'foo')
        self.assertRaises(KeyError,c.get,'x')
        c.exit()

class StencilTests(unittest.TestCase):
    '''
    Tests for the Stencil class
    '''

    def test_commit(self):
        s = Stencil()
        s.commit()

    def test_html(self):
        s = Stencil()
        s.html('Hello world')
        self.assertEqual(s.html().strip(),'Hello world')

    def renderCheck(self,inp,out,context=None):
        '''
        A shortcut method for testing renderin
        '''
        if context is None: context = Context()
        if type(context) is dict: context = Context(context)
        return self.assertEqual(
            Stencil().html(inp).render(context).html().strip(),
            out
        )

    def test_render_code(self):
        self.renderCheck(
            '<code data-code="python">a=1.2 ; b=3.4 ; c=a+b</code><div data-text="c"></div',
            '<code data-code="python">a=1.2 ; b=3.4 ; c=a+b</code><div data-text="c">4.6</div>'
        )

    def test_render_text(self):
        for value, text in (
            (1,'1'),
            (1.23,'1.23'),
            ('c','c'),
            ((1,2,3),'(1, 2, 3)'),
            ([1,2,3],'[1, 2, 3]'),
            ({'a':1,'b':2,'c':3},"{'a': 1, 'c': 3, 'b': 2}"),
        ):
            self.renderCheck(
                '<div data-text="value"></div>',
                '<div data-text="value">%s</div>'%text,
                {'value':value}
            )

if __name__ == '__main__':
    unittest.main()
    