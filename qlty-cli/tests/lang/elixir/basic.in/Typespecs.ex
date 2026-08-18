# Regression guard: declarations are not calls. Type declarations (@spec, @callback,
# @type), module attribute names, and the head of a defdelegate are all declarations, so
# none of them may contribute to cyclomatic complexity even when the name they mention
# collides with an entry in `iterator_method_identifiers`.
#
# Every colliding name below is deliberate: `map`, `filter`, `reduce`, `each` and the
# builtin type `map()` are all in that list. The module has zero executable branching, so
# its reported cyclomatic value must be exactly 1. Each construct that would break it:
#
#   @reduce                             attribute *name* collides
#   @callback filter(...)               callback declaration name collides
#   @spec map(list()) :: list()         spec declaration name collides
#   @spec fetch(map()) :: term()        nested type in ARGUMENT position collides
#   @spec merge(map(), map()) ...       the same nested type twice more
#   @type entry :: %{a: map()}          nested type inside a @type collides
#   defdelegate each(...), to: Enum     delegated function name collides
#   @type mapper :: (map() -> map())    nested type inside a FUNCTION TYPE collides
#   @callback fold(..., (map(), map() -> map()))  ditto, multi-argument function type
#   @spec transform((map() -> map()))   ditto, function type in argument position
#
# The function-type cases matter because an Elixir function type `(a -> b)` parses as a
# `stab_clause`, the same node kind used for case arms and `fn` clauses.
defmodule Typespecs do
  @reduce :configured

  @callback filter(list()) :: list()

  @callback fold(Enumerable.t(), (map(), map() -> map())) :: map()

  @type entry :: %{a: map()}

  @type mapper :: (map() -> map())

  defdelegate each(list, fun), to: Enum

  @spec map(list()) :: list()
  def map(list) do
    list
  end

  @spec fetch(map()) :: term()
  def fetch(m) do
    m
  end

  @spec merge(map(), map()) :: map()
  def merge(a, _b) do
    a
  end

  @spec transform((map() -> map())) :: mapper()
  def transform(fun) do
    fun
  end

  @spec strategy() :: atom()
  def strategy do
    @reduce
  end
end
