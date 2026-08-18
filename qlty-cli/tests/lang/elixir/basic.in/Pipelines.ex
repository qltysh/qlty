# Decision 7 end-to-end guard. Every function here is built from `=` bindings and
# `|>` pipelines through non-iterator functions, with no branching and no Enum or
# Stream call, so every reported cyclomatic value must be exactly 1. A value above 1
# means `binary_operator` was mapped wholesale into `binary_nodes`.
defmodule Pipelines do
  def normalize(input) do
    trimmed = String.trim(input)
    lowered = String.downcase(trimmed)
    lowered
  end

  def slug(input) do
    input
    |> String.trim()
    |> String.downcase()
    |> String.replace(" ", "-")
  end

  def full_path(root, name) do
    base = Path.join(root, name)

    base
    |> Path.expand()
    |> Kernel.to_string()
  end
end
