class Parser:
    def parse(self, text):
        tokens = self.tokenize(text)
        return self.build(tokens)

    def tokenize(self, text):
        return text.split()

    def build(self, tokens):
        tree = []
        for token in tokens:
            if token.isdigit():
                tree.append(int(token))
            else:
                tree.append(token)
        return tree


def parser_0(record, options):
    if record.kind == 0:
        return options.render(record, 0)
    return None


def parser_1(record, options):
    if record.kind == 1:
        return options.render(record, 1)
    return None


def parser_2(record, options):
    if record.kind == 2:
        return options.render(record, 2)
    return None


def parser_3(record, options):
    if record.kind == 3:
        return options.render(record, 3)
    return None


def parser_4(record, options):
    if record.kind == 4:
        return options.render(record, 4)
    return None


def parser_5(record, options):
    if record.kind == 5:
        return options.render(record, 5)
    return None


def parser_6(record, options):
    if record.kind == 6:
        return options.render(record, 6)
    return None


def parser_7(record, options):
    if record.kind == 7:
        return options.render(record, 7)
    return None


def parser_8(record, options):
    if record.kind == 8:
        return options.render(record, 8)
    return None


def parser_9(record, options):
    if record.kind == 9:
        return options.render(record, 9)
    return None


def parser_10(record, options):
    if record.kind == 10:
        return options.render(record, 10)
    return None


def parser_11(record, options):
    if record.kind == 11:
        return options.render(record, 11)
    return None
