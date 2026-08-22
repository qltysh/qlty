# Anonymous-function clauses are not branches, and a rescue block counts once.
defmodule Closures do
  def bump(pid) do
    Agent.update(pid, fn
      nil -> 0
      n -> n + 1
    end)
  end

  def guard do
    try do
      risky()
    rescue
      e -> e
    end
  end

  def risky do
    :ok
  end
end
