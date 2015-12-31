import stencila.extension as extension

from stencila.extension import Component

def grab(address):
    '''
    Grab a component from an address

    Resolves the local path from the address and
    the component type from the path.
    '''
    type = extension.type(address)
    if type == 'Stencil':
        from stencila.stencil import Stencil
        return Stencil(address)
    elif type == 'Theme':
        return extension.Theme(address)
    elif type == 'Sheet':
        from stencila.sheet import Sheet
        return Sheet(address)
    else:
        raise Exception(
            'Unhandled type at address:\n  type:%s\n  address:%s' %
            (type, address)
        )
