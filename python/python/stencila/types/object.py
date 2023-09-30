from typing import ForwardRef, Dict

Primitive = ForwardRef("Primitive")

Object = Dict[str, Primitive]
