import os

import stencila.extension as extension
from stencila.spread import Spread


class Sheet(extension.Sheet):

    def __init__(self, initialiser=None):
        if initialiser is not None:
            super(Sheet, self).__init__(initialiser)
        else:
            super(Sheet, self).__init__()
        self._spread = None
        self.attach()
        self.read_spread()

    def read_spread(self):
        if self._spread:
            pickle = os.path.join(self.path(), 'sheet.pkl')
            if os.path.exists(pickle):
                self._spread.read(pickle)

    def read(self, path="", base=True):
        super(Sheet, self).read(path)
        self.read_spread()

    def write_spread(self):
        if self._spread:
            pickle = os.path.join(self.path(), 'sheet.pkl')
            self._spread.write(pickle)

    def write(self, path):
        super(Sheet, self).write(path)
        self.write_spread()

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
