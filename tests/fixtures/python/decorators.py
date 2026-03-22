from dataclasses import dataclass
from pydantic import BaseModel


@dataclass
class Point:
    x: float
    y: float


class User(BaseModel):
    name: str
    email: str

    @staticmethod
    def helper() -> None:
        return None

    @classmethod
    def build(cls) -> "User":
        return cls(name="a", email="b@example.com")

    @property
    def value(self) -> int:
        return 42
