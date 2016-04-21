import stencila.extension as extension

from extension import Component

# List of component instances
# already instantiated
instances = {}


def instantiate(type, content, format):
    '''
    Instantiate a component

    This function is called by the C++ functions
    `Component::create` and `Component::get` to create a new instance
    '''
    type = type.lower()
    if type == 'stencil':
        from stencila.stencil import Stencil
        component = Stencil()
        if format == 'path':
            component.read(content)
        else:
            raise Exception('Unhandled instantiation format\n  format: ' + format)
    elif type == 'sheet':
        from stencila.sheet import Sheet
        component = Sheet()
        if format == 'path':
            component.read(content)
        elif format == 'json':
            component.read(content, 'json')
        else:
            raise Exception('Unhandled instantiation format\n  format: ' + format)
    else:
        raise Exception('Unhandled component type\n type:', type, '\n path:', path)

    global instances
    instances[component.address()] = component

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
