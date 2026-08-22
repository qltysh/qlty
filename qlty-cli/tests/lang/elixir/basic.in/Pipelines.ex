# `=` bindings and `|>` stages are not branches.
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
