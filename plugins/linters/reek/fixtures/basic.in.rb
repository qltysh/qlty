# My documented class
class Foo
  def check_response
    if respond_to?(:foo)
      puts "hi"
    elsif respond_to?(:bar)
      puts "var"
    end
  end
end
