import stencila.extension as extension

from stencila.extension import Component

def grab(address):
    '''
    Grab a component from an address

    Resolves the local path from the address and
    the component type from the path.
    '''
    type, path = extension.grab(address)
    if type == 'Stencil':
        from stencila.stencil import Stencil
        return Stencil(path)
    elif type == 'Theme':
        return extension.Theme(path)
    elif type == 'Sheet':
        from stencila.sheet import Sheet
        return Sheet(path)
    else:
        raise Exception(
            'Unhandled type at address:\n  type:%s\n  address:%s' %
            (type, address)
        )
