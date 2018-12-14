from stencilaschema.Processor import Processor


class TestProcessor(Processor):

    async def execute(self, thing, format='application/json', type=None):
        thing = await Processor.execute(self, thing, format, type)
        if thing.type == 'Cell':
            setattr(thing, 'executed', True) 
        return thing

class PersonProcessor(Processor):

    async def execute(self, thing, format='application/json', type=None):
        thing = await Processor.execute(self, thing, format, type)
        if thing.type == 'Person':
            given = getattr(thing, 'givenNames', [])
            family = getattr(thing, 'familyNames', [])
            setattr(thing, 'name', ' '.join(given + family))
        return thing
