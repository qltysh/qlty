# Enum calls are Elixir's iteration idiom, so each one is a decision point.
defmodule Iteration do
  def summarize(numbers) do
    numbers
    |> Enum.map(fn n -> n * 2 end)
    |> Enum.filter(fn n -> n > 4 end)
    |> Enum.reduce(0, fn n, acc -> n + acc end)
  end
end
