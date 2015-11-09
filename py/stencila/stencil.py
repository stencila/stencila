import stencila.extension as extension
from stencila.context import Context


class Stencil(extension.Stencil):

    def __init__(self, initialiser=None):
        if initialiser is not None:
            super(Stencil, self).__init__(initialiser)
        else:
            super(Stencil, self).__init__()

        self.attach()

    def context(self):
        if hasattr(self, '_context'):
            return self._context
        else:
            return None

    def attach(self, context=None):
        if hasattr(self, '_context') and self._context:
            self.detach()
        if context is None:
            context = Context()
        elif not isinstance(context, Context):
            context = Context(context)
        self._context = context
        super(Stencil, self).attach(context)
        return self

    def detach(self):
        self._context = None
        super(Stencil, self).detach()
        return self

    def render(self, context=None):
        if context is not None:
            self.attach(context)
        super(Stencil, self).render()
        return self
