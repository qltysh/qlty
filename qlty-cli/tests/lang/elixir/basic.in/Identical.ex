# The price/1 function below is copy-pasted verbatim into Identical2.ex. The two
# enclosing modules differ in shape, so the duplication engine reports the shared
# function itself as identical-code.
defmodule Identical do
  def price(order) do
    subtotal = order.quantity * order.unit_price
    discount = subtotal * order.discount_rate
    taxable = subtotal - discount
    tax = taxable * order.tax_rate
    shipping = order.shipping_base + order.weight * order.shipping_rate
    handling = order.handling_base
    total = taxable + tax + shipping + handling
    rounded = Float.round(total, 2)

    %{
      subtotal: subtotal,
      discount: discount,
      taxable: taxable,
      tax: tax,
      shipping: shipping,
      handling: handling,
      total: total,
      rounded: rounded
    }
  end
end
