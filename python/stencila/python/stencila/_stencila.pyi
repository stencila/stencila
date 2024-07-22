# SPDX-FileCopyrightText: 2024 Nokome Bentley
#
# SPDX-License-Identifier: Apache-2.0

from typing import TypedDict

class DecodeOptions(TypedDict):
    format: str | None

class EncodeOptions(TypedDict):
    format: str | None
    standalone: bool | None
    compact: bool | None

# This is actually a module.
class convert:  # noqa: N801
    @staticmethod
    async def from_string(string: str, options: DecodeOptions) -> str: ...
    @staticmethod
    async def from_path(path: str, options: DecodeOptions) -> str: ...
    @staticmethod
    async def to_string(json: str, options: EncodeOptions) -> str: ...
    @staticmethod
    async def to_path(json: str, path: str, options: EncodeOptions) -> str: ...
    @staticmethod
    async def from_to(
        input: str,  # noqa: A002
        output: str,
        decode_options: DecodeOptions,
        encode_options: EncodeOptions,
    ) -> str: ...
