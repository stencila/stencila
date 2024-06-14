import asyncio
import inspect
from functools import partial, wraps
from pathlib import Path

import typer
from rich import print

from .orm import Database, LLMCategory, ProviderRecord, RoutingRecord
from .provider import PROVIDERS, ProviderType
from .settings import get_settings
from .stats import CategoryResults, Routing, build_grid


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
async def dump(scrape_id: int):
    """Dump the JSON scraping record"""
    async with Database():
        rec = await ProviderRecord.filter(id=scrape_id).first()

    if rec is None:
        print(f"Record {scrape_id} not found.")
        raise typer.Exit(code=1)

    print(rec.dump)


@app.command()
async def show(table: str):
    """Show tables in the database."""
    from rich.table import Table

    if table == "provider":
        tb, flds = ProviderRecord, "id provider when".split()
    else:
        tb, flds = RoutingRecord, "id created provider_id".split()

    async with Database():
        records = await tb.all()

    # Create a rich table
    table = Table(title="Provider")

    # Add columns dynamically based on model fields
    for field in flds:
        table.add_column(field)

    # Add rows dynamically based on records
    for record in records:
        row = [str(getattr(record, field)) for field in flds]
        table.add_row(*row)

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
async def export(routing_id: int, output_path: Path):
    """Export a routing record to a JSON file."""
    async with Database():
        rec = await RoutingRecord.filter(id=routing_id).first()

        if rec is None:
            print(f"Record {routing_id} not found.")
            raise typer.Exit(code=1)

        categories = await rec.categories.all()

        results = []
        for c in categories:
            df = await build_grid(c.id)
            results.append(CategoryResults.from_grid(c.category, df))

        r = Routing(id=routing_id, categories=results)
    output_path.write_text(r.model_dump_json(indent=4))


@app.command()
async def settings():
    print(get_settings().model_dump())


def main():
    app()


if __name__ == "__main__":
    main()
