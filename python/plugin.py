# A very simple of a plugin

import json

from stencila import init, serve

def manifest(os: str, **kwargs) -> dict:
    return dict()

def dispatch(method: str, params: str) -> str:
    params = json.loads(request)

    def result():
        if method == "decode":
            input = params.get("input")
            format = params.get("from")
            if format == "json":
                return json.loads(input)

        raise RuntimeError("Incapable")
    
    return json.dumps(result())    

init(
    manifest, dispatch, log_level = "trace"
)

serve(
    protocol = "http",
    background = True
)
