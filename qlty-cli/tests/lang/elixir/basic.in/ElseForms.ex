# The two spellings of `if/else` are one branch and must score identically.
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
