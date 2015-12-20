import stencila.extension as extension


class Sheet(extension.Sheet):

    def __init__(self, initialiser=None):
        if initialiser is not None:
            super(Sheet, self).__init__(initialiser)
        else:
            super(Sheet, self).__init__()
