def main( ):
    x=1+2
    y = [ 1,2, 3 ]
    z={ 'a':1,'b':2 }
    
    if x==3:
        print('hello world')
    else   :
        print( 'goodbye'  )

import os,sys
import json

class    MyClass(   object ):
    def __init__(self,arg1,arg2):
        self.attr1=arg1
        self.attr2 = arg2

    def method_one( self,param ):
        result=param*2
        return result


def function_with_long_params(param1,param2,param3,param4,param5,param6):
    return param1+param2+param3+param4+param5+param6

# Bad string formatting
message = "This is a very long string that should probably be split across multiple lines but isn't and will exceed typical line length limits"

# Bad list formatting
long_list=[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20]

dict_example={'key1':'value1','key2':'value2','key3':'value3','key4':'value4'}