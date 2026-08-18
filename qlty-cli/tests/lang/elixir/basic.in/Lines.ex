# A module exercising the line metrics: comments, blank lines, a heredoc,
# a sigil and a charlist.
defmodule Lines do
  @moduledoc """
  A heredoc that spans
  several lines of prose.
  """

  # A leading comment.
  def greeting do
    "hello"
  end

  def words do
    ~w[alpha beta gamma]
  end

  # A trailing comment.
  def chars do
    ~c"abc"
  end
end
