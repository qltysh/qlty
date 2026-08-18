defmodule NestedControl do
  def dig(a, b, c) do
    if a do
      case b do
        1 ->
          if c do
            case a do
              2 ->
                if b do
                  :deep
                end

              _ ->
                :other
            end
          end

        _ ->
          :other
      end
    end
  end
end
