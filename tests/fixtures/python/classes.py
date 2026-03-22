class Animal:
    def __init__(self, name: str):
        self.name = name


class Dog(Animal):
    def bark(self) -> str:
        return "woof"
