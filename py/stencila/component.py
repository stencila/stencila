import stencila.extension as extension

from stencila.extension import Component

# List of component instances
# already instantiated
instances = {}


def instantiate(address, path, type):
    '''
    Instantiate a component

    This function is called by the C++ function
    `Component::get` to create a new instance
    '''
    if type == 'Stencil':
        from stencila.stencil import Stencil
        component = Stencil(path)
    elif type == 'Sheet':
        from stencila.sheet import Sheet
        component = Sheet(path)
    else:
        raise Exception('Unhandled component type\n type:', type, '\n path:', path)

    global instances
    instances[address] = component

    return component


def grab(address):
    '''
    Grab a component

    This is functionally the same as the C++ function
    `Component::get` but first checks for a locally instantiated
    instance of the component.
    '''
    global instances
    if address not in instances:
        extension.grab(address)

    # Component should now be instantiated and stored in `instances`
    # so return it from there
    return instances[address]
