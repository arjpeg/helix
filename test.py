from typing import Any


def decorator(fn: Any, *args: Any, **kwargs: Any) -> Any:
    print("decorator called", fn)

    def wrapper(*args: Any, **kwargs: Any) -> Any:
        # increase the context
        print("wrapper called", fn)
        args[0].context += 1
        res = fn(*args, **kwargs)
        args[0].context -= 1
        print("wrapper finished", args[0].context)

        return res

    return wrapper


class A:
    def __init__(self) -> None:
        self.context = 1

    @decorator
    def test(self) -> None:
        print("test called", self.context)


a = A()

a.test()
