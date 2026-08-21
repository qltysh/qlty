# Class, function and field counts; @moduledoc/@doc/@spec are excluded as reserved.
defmodule Members do
  @moduledoc "A module exercising class, function and field counts."

  @timeout 5000
  @retries 3

  defstruct [:id, :name]

  @doc "Starts the thing."
  @spec start() :: atom()
  def start do
    helper()
  end

  def stop do
    helper()
  end

  def helper do
    :ok
  end

  def external do
    Other.thing()
  end
end
