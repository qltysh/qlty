# Decision 11 end-to-end guard. summarize/1 pipes through Enum.map/2, Enum.filter/2
# and Enum.reduce/3 with single-clause anonymous functions and no other branching, so
# its reported cyclomatic value must be exactly 4: the base plus one per iteration
# call, with the `fn` clauses contributing nothing. A value of 1 means
# `iterator_method_identifiers()` was left empty and Elixir cyclomatic complexity is
# blind to the language's dominant iteration idiom.
defmodule Iteration do
  def summarize(numbers) do
    numbers
    |> Enum.map(fn n -> n * 2 end)
    |> Enum.filter(fn n -> n > 4 end)
    |> Enum.reduce(0, fn n, acc -> n + acc end)
  end
end
