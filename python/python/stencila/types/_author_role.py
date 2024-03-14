# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._author_role_name import AuthorRoleName
from ._person_or_organization_or_software_application import PersonOrOrganizationOrSoftwareApplication
from ._role import Role
from ._timestamp import Timestamp


@dataclass(init=False)
class AuthorRole(Role):
    """
    An author and their role.
    """

    type: Literal["AuthorRole"] = field(default="AuthorRole", init=False)

    author: PersonOrOrganizationOrSoftwareApplication
    """The author."""

    role_name: AuthorRoleName
    """A role played by the author."""

    last_modified: Optional[Timestamp] = None
    """Timestamp of most recent modification by the author in the role."""

    def __init__(self, author: PersonOrOrganizationOrSoftwareApplication, role_name: AuthorRoleName, id: Optional[str] = None, last_modified: Optional[Timestamp] = None):
        super().__init__(id = id)
        self.author = author
        self.role_name = role_name
        self.last_modified = last_modified
