# Decision 10 end-to-end guard.
#   bump/1  -> cyclomatic 1: Agent.update/2 is deliberately not an iterator method,
#              and the two clauses of the anonymous function contribute nothing,
#              matching every other language where closures do not branch.
#   guard/0 -> cyclomatic 3: base + the `try` expression + the `rescue` block. The
#              handler's stab_clause contributes nothing, matching Java's
#              `try { } catch (E e) { }`.
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
