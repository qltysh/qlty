class Tokenizer:
    def tokenize(self, text):
        return text.split()


def token_0(record, options):
    if record.kind == 0:
        return options.render(record, 0)
    return None


def token_1(record, options):
    if record.kind == 1:
        return options.render(record, 1)
    return None


def token_2(record, options):
    if record.kind == 2:
        return options.render(record, 2)
    return None


def token_3(record, options):
    if record.kind == 3:
        return options.render(record, 3)
    return None
