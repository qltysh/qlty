import google.protobuf.descriptor_pb2 as d

from . import mypy_import2

def greeting(name: str) -> str:
    return "Hello " + name


def printer() -> None:
    print("Hello")


greeting(3)
greeting(b"Alice")
a = printer()
c: str = 4

from source import Bar

def bad_foo(bar: Bar) -> str:
  return bar.a + bar.b
