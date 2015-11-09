import code
import sys
import os
import datetime

PY_VERSION = sys.version_info[0]

if PY_VERSION == 2:
    from cStringIO import StringIO
elif PY_VERSION == 3:
    from io import StringIO

# Create an `exec_` function that can be used in both
# Python 2 and 3. This is inspired by the exec_ function implmented in
# https://pypi.python.org/pypi/six but simplified for this use case
if PY_VERSION == 2:
    # Function signature compatible with Python 3's
    def exec_(code, globals, locals):
        assert globals is None
        exec("""exec code in {}, locals""")
elif PY_VERSION == 3:
    import builtins
    # Need to use `getattr` to avoid Python 2 syntax error
    exec_ = getattr(builtins, 'exec')

# Creat a `next` function as in Python 3
if PY_VERSION == 2:
    def next(iterator):
        return iterator.next()

# Attempt to load image packages
IMAGE_PACKAGE = None
try:
    import matplotlib
except ImportError:
    pass
else:
    IMAGE_PACKAGE = 'matplotlib'
    # Use the cairo backend because it support a variety of image formats.
    matplotlib.use('cairo')
    # Only once backend has been set can pylab be imported
    import pylab


class Context:
    '''
    A Python context

    Implements the sematics of the Context abstract base class for Python
    '''

    def __init__(self, scope=None):
        if scope is None:
            scope = Scope()
        self.scopes = [scope]

    # Shortcut methods for accessing the scope stack

    def push(self, scope=None):
        # Push a new scope on to the stack
        if scope is None:
            scope = Scope(self.top())
        self.scopes.append(scope)
        return self

    def pop(self):
        # Pop the current scope off the top of the stack
        self.scopes.pop(len(self.scopes)-1)
        return self

    def top(self):
        # Get the top of the namepace stack (i.e. the current scope)
        return self.scopes[len(self.scopes)-1]

    def get(self, name):
        # Get a variable from the top of the stack
        return self.top()[name]

    def set(self, name, value):
        # Set a variable in the top of the stack
        self.top()[name] = value
        return self

    def evaluate(self, expression):
        # Evaluate an expression in the top of the stack
        return eval(expression, {}, self.top())

    # Context methods that provide the interface defined in
    # cpp/stencila/context.hpp

    def execute(self, code, id="", format=None, width=None, height=None, units=None):
        # Some Python versions don't like trailing blank lines so remove all
        # surrounding whitepace
        code = code.strip()
        # Execute in top scope
        exec_(code, None, self.top())
        # Return according to format code
        if format in ('png', 'svg'):
            if not os.path.exists('out'):
                os.makedirs('out')
            filename = os.path.join('out', '%s.%s' % (id, format))
            if IMAGE_PACKAGE == 'matplotlib':
                pylab.savefig(filename)
            elif IMAGE_PACKAGE is None:
                raise Exception('No image rendering package is configured')
            else:
                raise Exception('Image rendering package not handled\n  package: %s' % IMAGE_PACKAGE)
            return filename
        else:
            return ""

    def interact(self, code):
        # Note that there is no buffering done here
        # since a new console in instantiated each time
        return Console(self.top()).interact(code+"\n")

    def assign(self, name, expression):
        self.top()[name] = self.evaluate(expression)

    def input(self, name, type, value):
        # Convert the string value to the appropriate Python type
        # Note that for text type there is no conversion,  the text value is
        # simply assigned to the variable
        # For a full list of input types see
        #   https://developer.mozilla.org/en-US/docs/Web/HTML/Element/Input
        if type == 'number':
            value = float(value)
        elif type == 'date':
            value = datetime.strptime(value, "%Y-%m-%d")
        elif type == 'datetime':
            value = datetime.strptime(value, "%Y-%m-%d %H:%M:%S")
        # Now assign the variable
        self.top()[name] = value

    def write(self, expression):
        return str(self.evaluate(expression))

    def test(self, expression):
        return bool(self.evaluate(expression))

    def mark(self, expression):
        self.enter()
        self.set('__subject__', self.evaluate(expression))

    def match(self, expression):
        try:
            subject = self.get('__subject__')
        except KeyError:
            raise Exception('No subject defined for switch directive')
        return subject == self.evaluate(expression)

    def unmark(self):
        self.exit()

    def begin(self, item, expression):
        self.enter()
        self.set('__item__', item)
        items = self.evaluate(expression)
        iterator = items.__iter__()
        self.set('__items__', iterator)
        return self.next()

    def next(self):
        try:
            item = next(self.get('__items__'))
        except StopIteration:
            return False
        else:
            self.set(self.get('__item__'), item)
            return True

    def enter(self, expression=""):
        if len(expression) > 0:
            self.push(self.evaluate(expression))
        else:
            self.push()

    def exit(self):
        self.pop()

    # Experimental; not used currently
    if 0:
        def bind(self, callback):
            '''
            Bind this Python side object to the C++ side
            '''
            self.set("__callback__", callback)


class Scope(dict):
    '''
    A variable scope within a Context
    '''

    def __init__(self, parent=None):
        self.parent = parent

    def __getitem__(self, name):
        try:
            return dict.__getitem__(self, name)
        except KeyError:
            if self.parent:
                return self.parent[name]
            else:
                raise KeyError(name)


class Console(code.InteractiveConsole):
    '''
    An interactive console within a Context

    Used for `interact` below
    See https://docs.python.org/2/library/code.html
    '''

    def __init__(self, locals):
        code.InteractiveConsole.__init__(self, locals)

    def push(self, source):
        # Push some source to the buffer
        # Redirect sys.stdout because that is what gets used for
        # both printing (i.e. via "print") and the "showing" of a value
        stdout = sys.stdout
        sys.stdout = StringIO()
        # Reset error string
        self.err = ""
        # Run code
        more = code.InteractiveConsole.push(self, source)
        # Get output and restore sys.stdout
        self.out = sys.stdout.getvalue()
        sys.stdout = stdout
        return more

    def write(self, line):
        # Capture a line of traceback (lines have a newline at end)
        self.err += line

    def interact(self, source):
        # Push some source and then concatenate output and errors
        # Note that currently di not use return value of push()
        self.push(source)
        return self.out+self.err
