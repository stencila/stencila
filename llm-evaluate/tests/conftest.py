import re
from pathlib import Path

import orjson
import pytest

from llm_evaluate.orm import close_connection, init_connection

HERE = Path(__file__).parent
DATA_PATH = HERE / "data"


@pytest.fixture(scope="session")
def data_path():
    return DATA_PATH


# This is basic Stencila Article template, with a single InstructionBlock
ARTICLE_TEMPLATE = """
{
  "type": "Article",
  "content": [
    {
      "type": "InstructionBlock",
      "messages": [
        {
          "type": "InstructionMessage",
          "parts": [
            {
              "type": "Text",
              "value": {
                "string": {question}
              }
            }
          ]
        }
      ],
      "assignee": "answer"
    }
  ]
}
"""


def generate_safe_filename(text: str) -> str:
    # Remove characters that are not alphanumeric, spaces, or underscores
    safe_text = re.sub(r"[^a-zA-Z0-9\s-]", "", text)
    # Replace spaces and hyphens with underscores
    safe_text = re.sub(r"[\s-]+", "_", safe_text)
    # Convert to lowercase
    safe_text = safe_text.lower()
    return safe_text


@pytest.fixture(scope="session")
def smd_path(tmp_path_factory):
    fake_path = DATA_PATH / "fake.json"
    questions = orjson.loads(fake_path.read_text())
    tmp_path = tmp_path_factory.mktemp("smd")
    for q in questions:
        q_text = q["question"]
        filename = generate_safe_filename(q_text)
        article = ARTICLE_TEMPLATE.replace("{question}", q_text)
        (tmp_path / f"{filename}.json").write_text(orjson.dumps(article).decode())

    return tmp_path


@pytest.fixture()
async def with_sqlite():
    # TODO: have a fixture that wraps connection.
    await init_connection("sqlite://:memory:")
    yield
    await close_connection()
