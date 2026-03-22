__all__ = ["exported_func", "ExportedClass"]


def exported_func() -> None:
    return None


class ExportedClass:
    pass


def hidden_func() -> None:
    return None
