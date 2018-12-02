from stencilaschema.Processor import Processor
from stencilaschema.types.Thing import Thing
from stencilaschema.types.utils import dehydrate

processor = Processor()

string1 = '{"name":"thing1"}'
obj1 = {"name": "thing1"}
thing1 = Thing(name="thing1")

def test_import():
    assert dehydrate(processor.import_(string1)) == obj1
    assert dehydrate(processor.import_(obj1)) == obj1
    assert dehydrate(processor.import_(thing1)) == obj1

def test_export():
    assert processor.export(thing1) == string1

def test_compile():
    assert dehydrate(processor.compile(thing1)) == obj1

def test_build():
    assert dehydrate(processor.build(thing1)) == obj1

def test_execute():
    assert dehydrate(processor.execute(thing1)) == obj1
