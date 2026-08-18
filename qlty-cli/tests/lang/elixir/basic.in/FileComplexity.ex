defmodule FileComplexity do
  def one(a, b) do
    if a do
      case b do
        1 -> if a, do: :x, else: :y
        _ -> if b, do: :x, else: :y
      end
    end
  end

  def two(a, b) do
    if a do
      case b do
        1 -> if a, do: :x, else: :y
        _ -> if b, do: :x, else: :y
      end
    end
  end

  def three(a, b) do
    if a do
      case b do
        1 -> if a, do: :x, else: :y
        _ -> if b, do: :x, else: :y
      end
    end
  end

  def four(a, b) do
    if a do
      case b do
        1 -> if a, do: :x, else: :y
        _ -> if b, do: :x, else: :y
      end
    end
  end

  def five(a, b) do
    if a do
      case b do
        1 -> if a, do: :x, else: :y
        _ -> if b, do: :x, else: :y
      end
    end
  end

  def six(a, b) do
    if a do
      case b do
        1 -> if a, do: :x, else: :y
        _ -> if b, do: :x, else: :y
      end
    end
  end
end
