defmodule FunctionComplexity do
  def process(items, flag, other) do
    for item <- items do
      if flag do
        case item do
          {:ok, value} ->
            if value > 0 and other do
              if value > 10 or flag do
                :big
              else
                :small
              end
            else
              :zero
            end

          {:error, reason} ->
            case reason do
              :timeout ->
                if other do
                  :retry
                else
                  :fail
                end

              _ ->
                :fail
            end

          _ ->
            :skip
        end
      else
        :disabled
      end
    end
  end
end
