# Declarations are not calls: every name here collides with an iterator method.
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
