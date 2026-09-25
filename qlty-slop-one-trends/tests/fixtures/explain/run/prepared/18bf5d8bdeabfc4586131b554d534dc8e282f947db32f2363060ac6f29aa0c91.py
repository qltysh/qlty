class Builder:
    def build(self, tokens):
        tree = []
        for token in tokens:
            if token.isdigit():
                tree.append(int(token))
            else:
                tree.append(token)
        return tree


def builder_0(record, options):
    if record.kind == 0:
        return options.render(record, 0)
    return None


def builder_1(record, options):
    if record.kind == 1:
        return options.render(record, 1)
    return None


def builder_2(record, options):
    if record.kind == 2:
        return options.render(record, 2)
    return None


def builder_3(record, options):
    if record.kind == 3:
        return options.render(record, 3)
    return None


def builder_4(record, options):
    if record.kind == 4:
        return options.render(record, 4)
    return None


def builder_5(record, options):
    if record.kind == 5:
        return options.render(record, 5)
    return None


def builder_6(record, options):
    if record.kind == 6:
        return options.render(record, 6)
    return None


def builder_7(record, options):
    if record.kind == 7:
        return options.render(record, 7)
    return None
