import extension


def grab(address):
    '''
    Grab a component from an address

    Resolves the local path from the address and
    the component type from the path.
    '''
    type, path = extension.grab(address)
    if type == 'Stencil':
        return extension.Stencil(path)
    elif type == 'Theme':
        return extension.Theme(path)
    else:
        raise Exception(
            'Unhandled type at address:\n  type:%s\n  address:%s' %
            (type, address)
        )
