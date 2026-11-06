interface Order {
  id: string;
  items: { price: number; quantity: number }[];
  discountCode?: string;
  shippingCountry: string;
}

export function calculateInvoiceTotal(order: Order): number {
  let subtotal = 0;
  for (const item of order.items) {
    if (item.quantity <= 0) {
      throw new Error(`Invalid quantity for order ${order.id}`);
    }
    subtotal += item.price * item.quantity;
  }

  let discount = 0;
  if (order.discountCode === "SAVE10") {
    discount = subtotal * 0.1;
  } else if (order.discountCode === "SAVE20") {
    discount = subtotal * 0.2;
  }

  let shipping = 0;
  if (order.shippingCountry === "US") {
    shipping = subtotal > 100 ? 0 : 9.99;
  } else if (order.shippingCountry === "CA") {
    shipping = subtotal > 150 ? 0 : 14.99;
  } else {
    shipping = 29.99;
  }

  const tax = (subtotal - discount) * 0.08;
  return Math.round((subtotal - discount + shipping + tax) * 100) / 100;
}

export function calculateQuoteTotal(order: Order): number {
  let subtotal = 0;
  for (const item of order.items) {
    if (item.quantity <= 0) {
      throw new Error(`Invalid quantity for order ${order.id}`);
    }
    subtotal += item.price * item.quantity;
  }

  let discount = 0;
  if (order.discountCode === "SAVE10") {
    discount = subtotal * 0.1;
  } else if (order.discountCode === "SAVE20") {
    discount = subtotal * 0.2;
  }

  let shipping = 0;
  if (order.shippingCountry === "US") {
    shipping = subtotal > 100 ? 0 : 9.99;
  } else if (order.shippingCountry === "CA") {
    shipping = subtotal > 150 ? 0 : 14.99;
  } else {
    shipping = 29.99;
  }

  const tax = (subtotal - discount) * 0.08;
  return Math.round((subtotal - discount + shipping + tax) * 100) / 100;
}
