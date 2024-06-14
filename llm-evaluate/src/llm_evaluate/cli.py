import asyncio
import inspect
from functools import partial, wraps

import typer
from rich import print

from .orm import Database, ProviderRecord, LLMCategory, RoutingRecord
from .provider import PROVIDERS, ProviderType
from .settings import get_settings


# A hack for enabling async functions in typer
# https://github.com/tiangolo/typer/issues/88
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


app = AsyncTyper(
    add_completion=False,
    no_args_is_help=True,
    help="lemmy: Evaluate LLM providers",
)


@app.command()
async def scrape(provider: ProviderType):
    """Scrape a provider, saving the results to the database."""
    if provider not in PROVIDERS:
        print(f"Provider {provider} not found.")
        raise typer.Exit(code=1)

    provider_cls = PROVIDERS[provider]
    scraped = await provider_cls.scrape()

    # print(scraped.model_dump())

    async with Database():
        rec = scraped.to_provider_record()
        await rec.save()

    print("Done!")


@app.command()
async def history():
    """Show the history of provider (scraped) records."""
    from rich.table import Table

    async with Database():
        records = await ProviderRecord.all()

    # Create a rich table
    table = Table(title="Records")

    # Add columns dynamically based on model fields
    fields = "id provider when".split()
    for field in fields:
        table.add_column(field)

    # Add rows dynamically based on records
    for record in records:
        row = [str(getattr(record, field)) for field in fields]
        table.add_row(*row)

    # Create a console and display the table
    print(table)



@app.command()
async def generate(scrape_id: int):
    """Generate a routing record from a scrape record."""
    async with Database():
        rec = await ProviderRecord.filter(id=scrape_id).first()

    if rec is None:
        print(f"Record {scrape_id} not found.")
        raise typer.Exit(code=1)

    provider_cls = PROVIDERS[rec.provider]
    scraped = provider_cls.model_validate(rec.dump)

    async with Database():
        routing = await RoutingRecord.create(provider=rec)
        for category in LLMCategory:
            await scraped.generate_snapshot(routing, category)



@app.command()
async def settings():
    print(get_settings().model_dump())


def main():
    app()


if __name__ == "__main__":
    main()
