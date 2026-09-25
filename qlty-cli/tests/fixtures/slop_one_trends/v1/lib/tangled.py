def route(order, user, region, currency, discount, priority, notes):
    if order:
        if user:
            if region:
                if currency:
                    if discount:
                        if priority and notes and region == "eu" and currency == "eur":
                            return 1
                        return 2
                    return 3
                return 4
            return 5
        return 6
    return 7


def price_a(items, rate):
    subtotal = 0
    for item in items:
        if item.taxable:
            subtotal += item.price * (1 + rate)
        else:
            subtotal += item.price
    if subtotal > 100:
        subtotal -= 5
    if subtotal > 200:
        subtotal -= 10
    if subtotal > 300:
        subtotal -= 15
    if subtotal > 400:
        subtotal -= 20
    if subtotal > 500:
        subtotal -= 25
    return round(subtotal, 2)


def price_b(items, rate):
    subtotal = 0
    for item in items:
        if item.taxable:
            subtotal += item.price * (1 + rate)
        else:
            subtotal += item.price
    if subtotal > 100:
        subtotal -= 5
    if subtotal > 200:
        subtotal -= 10
    if subtotal > 300:
        subtotal -= 15
    if subtotal > 400:
        subtotal -= 20
    if subtotal > 500:
        subtotal -= 25
    return round(subtotal, 2)
