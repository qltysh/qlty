# Hand-derived expected metrics for this file (Decision 3, 4 and 6):
#   classes   = 1  (one defmodule; a module is the unit of "class")
#   functions = 4  (start/0, stop/0, helper/0, external/0)
#   fields    = 4  (@timeout, @retries, plus the :id and :name defstruct fields;
#                   @moduledoc, @doc and @spec are reserved and excluded)
#   lcom4     = 1  (start -> {start, helper} and stop -> {stop, helper} both survive
#                   and merge on helper; helper -> {helper} and external -> {external}
#                   are each dropped by the "no fields and <= 1 function" rule, since
#                   the qualified Other.thing/0 call is not an intra-module reference)
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
