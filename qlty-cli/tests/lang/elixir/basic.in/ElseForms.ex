# `if x, do: a, else: b` and `if x do a else b end` are one branch written two ways, so
# they must contribute identical complexity. The keyword form's `else:` is a `pair` in the
# conditional's argument list rather than an `else_block`, and the more idiomatic of the
# two spellings, so a classifier that only knows `else_block` silently halves the
# cognitive complexity of ordinary Elixir. Hand-derived expectations:
#
#   keyword/1  cognitive 2 (if + else), cyclomatic +1 (an else is not a decision point)
#   block/1    cognitive 2 (if + else), cyclomatic +1
#
#   file cyclomatic = 1 + 1 + 1 = 3
#   file cognitive  =     2 + 2 = 4
#
# The two functions must report the same values as each other; a divergence here is the
# regression this file exists to catch.
defmodule ElseForms do
  def keyword(x) do
    if x, do: :yes, else: :no
  end

  def block(x) do
    if x do
      :yes
    else
      :no
    end
  end
end
