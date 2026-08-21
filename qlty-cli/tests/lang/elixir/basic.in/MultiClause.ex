# Clause heads are decision points. Idiomatic Elixir pushes branching out of function
# bodies and into the heads, so a module of multi-clause functions must not report the
# same complexity as a module of straight-line ones. Hand-derived expectations:
#
#   route/1        4 clauses -> cyclomatic +3, cognitive +1
#   describe/1     3 clauses -> cyclomatic +2, cognitive +1
#   passthrough/1  1 clause  -> +0
#
#   file cyclomatic = 1 + 3 + 2 = 6
#   file cognitive  =     1 + 1 = 2
#
# A clause group counts once toward cognitive complexity, matching the single `case` it
# replaces, and once per clause after the first toward cyclomatic, one less than the
# equivalent `case` because the final clause is the fallthrough rather than a tested arm.
#
# `functions` is 8, not 3: each clause is its own definition. Clause groups are
# deliberately not collapsed, so this file also pins that counting decision.
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
