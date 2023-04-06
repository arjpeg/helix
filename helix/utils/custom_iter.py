from typing import Generic, Iterator, TypeVar

T = TypeVar("T")


class CustomIter(Generic[T]):
    def __init__(self, iterator: Iterator[T]):
        self.iterator = iterator
        self.history: list[T | None] = [
            None,
        ]
        self.i = 0

    def next(self):
        self.i += 1
        if self.i < len(self.history):
            return self.history[self.i]
        else:
            elem = next(self.iterator)
            self.history.append(elem)
            return elem

    def prev(self):
        self.i -= 1
        if self.i == 0:
            raise StopIteration
        else:
            return self.history[self.i]
