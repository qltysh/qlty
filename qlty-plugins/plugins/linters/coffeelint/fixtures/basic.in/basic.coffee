# Both of this will not trigger an error,
# even with arrow_spacing enabled.
x(-> 3)
x( -> 3)

# However, this will trigger an error
x((a,b)-> 3)

# Good!
class BoaConstrictor

# Bad!
class boaConstrictor
