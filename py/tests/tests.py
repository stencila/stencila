import unittest

import stencila


class StencilaTests(unittest.TestCase):
    
    def test_version(self):
        stencila.version()

class ExceptionTests(unittest.TestCase):

    def test_translation(self):
        try:
            stencila.extension.exception_test()
        except:
            pass
        else:
            raise

class ComponentTests:

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


    def test_path(self):
        self.assertRegexpMatches(self.component.path("").path(),"~")

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



class PackageTests(ComponentTests,unittest.TestCase):
    
    def setUp(self):
        self.component = stencila.Package()

    def tearDown(self):
        self.component.destroy()


if __name__ == '__main__':
    unittest.main()
    