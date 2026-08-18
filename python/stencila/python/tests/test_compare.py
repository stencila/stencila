"""
Tests of functions in the `compare` module
"""

import pytest
from stencila_types import types as T

from stencila.compare import (
    CompareError,
    compare_nodes,
    compare_paths,
    compare_strings,
    is_equal,
    report_paths,
    report_strings,
)

PARAGRAPH = "../../examples/conversion/paragraph/paragraph.json"


def test_equal_documents_have_no_differences():
    comparison = compare_strings(
        "Hello world", "Hello world", left_format="md", right_format="md"
    )

    assert comparison["differences"] == []


def test_changed_text_is_a_value_difference():
    comparison = compare_strings(
        "Hello world", "Hello there", left_format="md", right_format="md"
    )

    kinds = [difference["type"] for difference in comparison["differences"]]
    assert "valueChanged" in kinds


def test_nodes_can_be_compared_in_memory():
    left = T.Article(content=[T.Paragraph(content=[T.Text(value="Hello world")])])
    right = T.Article(content=[T.Paragraph(content=[T.Text(value="Hello there")])])

    assert compare_nodes(left, left)["differences"] == []
    assert compare_nodes(left, right)["differences"] != []


def test_wholly_unrecognizable_content_is_one_sided_not_different():
    """A paragraph with nothing in common is unmatched, so there is no difference to
    report about it -- only a one-sided correspondence. Equality has to account for
    both, which is what `is_equal` is for."""
    left = T.Article(content=[T.Paragraph(content=[T.Text(value="One")])])
    right = T.Article(content=[T.Paragraph(content=[T.Text(value="Two")])])

    comparison = compare_nodes(left, right)
    assert comparison["differences"] == []

    kinds = {
        correspondence["type"]
        for correspondence in comparison["alignment"]["correspondences"]
    }
    assert "leftOnly" in kinds
    assert "rightOnly" in kinds
    assert not is_equal(comparison)


def test_is_equal_follows_the_comparison():
    equal = compare_strings("Hello", "Hello", left_format="md", right_format="md")
    assert is_equal(equal)

    different = compare_strings("Hello", "Goodbye", left_format="md", right_format="md")
    assert not is_equal(different)


def test_a_document_is_equal_to_itself():
    comparison = compare_paths(PARAGRAPH, PARAGRAPH)

    assert comparison["differences"] == []


def test_exclude_suppresses_differences():
    left = T.Article(id="one", content=[T.Paragraph(content=[T.Text(value="Hi")])])
    right = T.Article(id="two", content=[T.Paragraph(content=[T.Text(value="Hi")])])

    assert compare_nodes(left, right)["differences"] != []

    filtered = compare_nodes(left, right, exclude=["id"])
    assert filtered["differences"] == []
    assert filtered["suppressedDifferences"] > 0
    assert is_equal(filtered)


def test_unknown_selectors_are_rejected():
    with pytest.raises(ValueError, match="not a property in the Stencila Schema"):
        compare_strings(
            "One", "Two", left_format="md", right_format="md", exclude=["nope"]
        )


def test_a_missing_document_is_an_error():
    with pytest.raises(CompareError):
        compare_paths("does-not-exist.smd", PARAGRAPH)


def test_text_report_names_both_sides():
    report = report_strings(
        "Hello world",
        "Hello there",
        left_format="md",
        right_format="md",
        left_label="before.md",
        right_label="after.md",
    )

    assert report.startswith("different\n")
    assert "left:  before.md" in report
    assert "right: after.md" in report


def test_html_report_is_self_contained():
    report = report_strings(
        "Hello world", "Hello there", left_format="md", right_format="md", format="html"
    )

    assert report.startswith("<!DOCTYPE html>")
    assert "<style>" in report
    assert "<script" not in report


def test_path_reports_are_labelled_with_their_paths():
    report = report_paths(PARAGRAPH, PARAGRAPH)

    assert report.startswith("equal\n")
    assert PARAGRAPH in report


def test_unknown_report_formats_are_rejected():
    with pytest.raises(ValueError, match="format must be"):
        report_strings("One", "Two", format="pdf")  # type: ignore[arg-type]
