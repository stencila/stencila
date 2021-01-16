import json

from stencila import init, serve


def manifest(os: str, **kwargs) -> dict:
    return {
        "methods": {
            "execute": {
                "node": {
                    "type": {"const": "CodeExpression"},
                    "programmingLanguage": {"enum": ["py", "python", "python3"]},
                }
            }
        }
    }


def dispatch(method: str, params: str) -> str:
    params = json.loads(params)

    def result():
        if method == "execute":
            node = params.get("node")
            if node.get("type") == "CodeExpression" and node.get(
                "programmingLanguage"
            ) in [
                "py",
                "python",
                "python3",
            ]:
                return eval(node.get("text"))

        raise RuntimeError("Unable to handle request")

    return json.dumps(result())


# Initialize the plugin by registering its manifest and
# dispatch functions
init(manifest, dispatch)

# Serve the plugin over standard I/O
serve()
