import ast
import importlib
import pickle
import re


class Spread:
    '''
    A spread for a sheet

    Spreads are to Sheets what Contexts are to Stencils.
    The spread is attached to a sheet and provides it's evaluation environment
    '''

    def __init__(self):
        self.packages = {}
        self.variables = {}

    def _eval(self, expression):
        '''
        Evaluate an expression within the spread environment.
        Private method used in methods below.
        '''
        try:
            value = eval(
                expression,
                self.packages,
                self.variables
            )
        except Exception, exc:
            value = exc
        return value

    def _content(self, value):
        '''
        Get the type and string representation of a value
        Private method used in methods below.
        '''
        if isinstance(value,Exception):
            return ('error', str(value))
        else:
            tipe = type(value).__name__
            if tipe == 'bool':
                tipe = 'boolean'
            elif tipe == 'int':
                tipe = 'integer'
            elif tipe == 'float':
                tipe = 'real'
            elif tipe == 'str':
                tipe = 'string'
            rep = repr(value)
            return (tipe, rep)

    # Following method implement the `Spread` interface

    _import_regex = re.compile('import +(\w+)')

    def execute(self, source):
        match = self._import_regex.match(source)
        if match:
            package = match.group(1)
            self.packages[package] = __import__(package)
            return 'import ' + package
        else:
            return ''

    def evaluate(self, expression):
        value = self._eval(expression)
        content = self._content(value)
        return ' '.join(content)

    def set(self, id, expression, name):
        value = self._eval(expression)
        self.variables[id] = value
        if name != "":
            self.variables[name] = value
        content = self._content(value)
        return ' '.join(content)

    def get(self, name):
        value = self.variables[name]
        return self._content(value)

    def clear(self, id="", name=""):
        if id:
            del self.variables[id]
            if name:
                del self.variables[id]
        else:
            self.variables = {}

    def list(self):
        return ','.join(self.variables.keys())

    def depends(self, expression):
        collector = SpreadNameCollector()
        collector.visit(
            ast.parse(expression)
        )
        return ','.join(collector.names)

    def read(self, path):
        pickle.load(path)

    def write(self, path):
        pass


class SpreadNameCollector(ast.NodeVisitor):

    def __init__(self):
        self.names = []

    def visit_Name(self, node):
        self.names.append(node.id)
