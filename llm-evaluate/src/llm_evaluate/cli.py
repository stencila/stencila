import asyncio
import inspect
from functools import partial, wraps

import typer
from rich import print

from .provider import PROVIDERS


class AsyncTyper(typer.Typer):
    @staticmethod
    def maybe_run_async(decorator, f):
        if inspect.iscoroutinefunction(f):

            @wraps(f)
            def runner(*args, **kwargs):
                return asyncio.run(f(*args, **kwargs))

            decorator(runner)
        else:
            decorator(f)
        return f

    def callback(self, *args, **kwargs):
        decorator = super().callback(*args, **kwargs)
        return partial(self.maybe_run_async, decorator)

    def command(self, *args, **kwargs):
        decorator = super().command(*args, **kwargs)
        return partial(self.maybe_run_async, decorator)


app = AsyncTyper()


@app.command()
async def scrape(provider: str):
    if provider not in PROVIDERS:
        print(f"Provider {provider} not found.")
        raise typer.Exit(code=1)

    provider_cls = PROVIDERS[provider]
    scraped = await provider_cls.scrape()
    print(scraped.model_dump())


@app.command()
async def test():
    typer.echo("Hello World")


def main():
    app()


if __name__ == "__main__":
    main()
