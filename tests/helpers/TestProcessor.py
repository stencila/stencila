from stencilaschema.Processor import Processor


class TestProcessor(Processor):

    async def execute(self, thing, format='application/json', type=None):
        thing = await Processor.execute(self, thing, format, type)
        if thing.type == 'Cell':
            setattr(thing, 'executed', True) 
        return thing
