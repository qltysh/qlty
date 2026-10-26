defmodule Script do
  def run(mode) do
    if mode == :verbose do
      report(:verbose)
    else
      report(:quiet)
    end
  end

  def report(level) do
    level
  end
end

Script.run(:quiet)
