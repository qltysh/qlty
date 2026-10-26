# Elixir has no `return`: this branchy module must report no return-statements issues.
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
