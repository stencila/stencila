import os

import stencila.extension as extension
from stencila.spread import Spread


class Sheet(extension.Sheet):

    def __init__(self, initialiser=None):
        super(Sheet, self).__init__()
        self.attach()
        if initialiser is not None:
            self.initialise(initialiser)

    def spread(self):
        if hasattr(self, '_spread'):
            return self._spread
        else:
            return None

    def attach(self, spread=None):
        if hasattr(self, '_spread') and self._spread:
            self.detach()
        if spread is None:
            spread = Spread()
        elif not isinstance(spread, Spread):
            spread = Spread(spread)
        self._spread = spread
        super(Sheet, self).attach(spread)
        return self

    def detach(self):
        self._spread = None
        super(Sheet, self).detach()
        return self
