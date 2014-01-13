import unittest

import stencila

class Stencila(unittest.TestCase):
    
    def test_version(self):
        stencila.version()

if __name__ == '__main__':
    unittest.main()
    