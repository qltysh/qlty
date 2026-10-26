# Clause heads are decision points: route/1 has 4 clauses, describe/1 has 3 guarded.
defmodule MultiClause do
  def route(:get), do: :read
  def route(:put), do: :write
  def route(:post), do: :write
  def route(_), do: :unknown

  def describe(n) when n < 0, do: :negative
  def describe(n) when n > 0, do: :positive
  def describe(_), do: :zero

  def passthrough(x), do: x
end
