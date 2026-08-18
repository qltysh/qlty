# Decision 5: Elixir has no `return` keyword, so `return_nodes()` is empty and the
# `return-statements` smell can never fire. This module is deliberately branchy: it
# has many exit points expressed as case/cond arms, and must still produce zero
# `return-statements` issues.
defmodule Returns do
  def classify(value) do
    case value do
      0 -> :zero
      1 -> :one
      2 -> :two
      3 -> :three
      4 -> :four
      5 -> :five
      _ -> :many
    end
  end

  def describe(value) do
    cond do
      value < 0 -> :negative
      value == 0 -> :zero
      value < 10 -> :small
      value < 100 -> :medium
      value < 1000 -> :large
      true -> :huge
    end
  end
end
