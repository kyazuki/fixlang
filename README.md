Fix-lang
====

## Overview

Fix is a programming language with the following features: 
- Functional: All functions have no side effect and all values are immutable. This reduces bugs caused by state management failures.
- O(1) update of arrays and structures: Despite the 1st feature, Fix mutates a value if the mutation cannot be observed. For example, `let array1 = array0.set(10, 42);` defines a new array `array1` that is almost identical to `array0` but with the 10th element replaced by 42. If `array0` will not be referenced later, Fix will update the 10th element of `array0` and rename it as `array1`. On the other hand, if `array0` may be used later, Fix creates `array1` by cloning `array0` and setting the 10th element to 42, keeping immutability.
- Familier syntax: The syntax of Fix is more similar to languages such as C++ or Rust than to other functional languages such as Haskell. Even if you have never learned a functional language, you will be able to learn Fix quickly.

In another perspective, Fix is a language which uses reference counting to provide garbage collection and interior mutability. To avoid circular reference, all values are semantically immutable and it restricts dynamic recursive definition and forces to use fixed-point combinator instead. To reduce copy cost on "modify" operation of a value, Fix mutates it if the reference counter is one.

You can try Fix in [fixlang playground](https://tttmmmyyyy.github.io/fixlang-playground/).

(This project is still a WIP and has no practical use yet.)

## Examples

- [Basic syntax](https://tttmmmyyyy.github.io/fixlang-playground/index.html?src=%2F%2F+Each+source+file+has+to+start+with+module+declaration.%0D%0Amodule+Main%3B%0D%0A%0D%0A%2F%2F+Declaration+and+definition+of+global+value.%0D%0A%2F%2F+%60I64%60+is+the+type+of+64-bit+integers.%0D%0Atruth+%3A+I64%3B%0D%0Atruth+%3D+42%3B+%0D%0A%0D%0A%2F%2F+Declaration+and+definition+of+global+%28recursive%29+function.%0D%0A%2F%2F+To+define+function%2C+write+%60%7Carg0%2C+arg1%2C+...%7C+%28function+body%29%60.%0D%0A%2F%2F+%28Parentheses+around+%60%28function+body%29%60+is+not+mandatory.%29%0D%0A%2F%2F+Note+that+Fix+is+an+expression+based+language.+You+don%27t+need+to+write+%22return+statement%22.%0D%0Acalc_fib+%3A+I64+-%3E+I64%3B%0D%0Acalc_fib+%3D+%7Cn%7C+%28%0D%0A++++if+n+%3C%3D+1+%7B+n+%7D+else+%7B+calc_fib%28n-1%29+%2B+calc_fib%28n-2%29+%7D%0D%0A%29%3B%0D%0A%0D%0Acalc_fib2+%3A+I64+-%3E+I64%3B%0D%0Acalc_fib2+%3D+%7Cn%7C+%28%0D%0A++++%2F%2F+Another+syntax+of+%60if%60%2C+%60if+%28cond%29+%7B+%28then+expr%29+%7D%3B+%28else+expr%29%60%2C+can+be+used+to+write+early+return.%0D%0A++++if+n+%3C%3D+1+%7B+n+%7D%3B%0D%0A%0D%0A++++%2F%2F+Use+%60let%60+to+define+a+local+name.%0D%0A++++let+x+%3D+calc_fib2%28n-1%29%3B%0D%0A++++let+y+%3D+calc_fib2%28n-2%29%3B%0D%0A++++x+%2B+y%0D%0A%29%3B%0D%0A%0D%0Atruth2+%3A+I64%3B%0D%0Atruth2+%3D+%28%0D%0A++++%2F%2F+You+can+define+local+function+%28closure%29+like+this.+%60f%60+has+type+%60I64+-%3E+I64+-%3E+I64+-%3E+I64%60.%0D%0A++++let+f+%3D+%7Ca%2C+b%2C+c%7C+%28a+%2B+b%29+%2A+c%3B%0D%0A%0D%0A++++%2F%2F+Partial+application.+%60double%60+has+type+%60I64+-%3E+I64%60+and+maps+%60c%60+to+%60%281+%2B+1%29+%2A+c+%3D%3D+2+%2A+c%60.%0D%0A++++let+double+%3D+f%281%2C+1%29%3B%0D%0A%0D%0A++++%2F%2F+Right-associative+operator+%60%24%60+applies+a+function+to+a+value%3A+%60f+%24+x+%3D%3D+f%28x%29%60+and+%60f+%24+g+%24+x+%3D%3D+f%28g%28x%29%29%60.%0D%0A++++let+twelve+%3D+double+%24+double+%24+3%3B%0D%0A%0D%0A++++%2F%2F+%60.%60+is+another+operator+to+apply+a+function%3A+%60x.f+%3D%3D+f%28x%29%60.%0D%0A++++%2F%2F+It+has+lower+priority+than+usual+function+call%2C+so+%603.f%281%2C+2%29+%3D%3D+f%281%2C+2%29%283%29+%3D%3D+f%281%2C+2%2C+3%29%60.%0D%0A++++let+nine+%3D+3.f%281%2C+2%29%3B%0D%0A%0D%0A++++double+%24+nine+%2B+twelve%0D%0A%29%3B%0D%0A%0D%0A%2F%2F+Fix+program+calls+%60Main%3A%3Amain%60+%28i.e.%2C+%60main%60+of+%60Main%60+module%29+as+the+entry+point.%0D%0A%2F%2F+%60Main%3A%3Amain%60+must+have+type+%60IO+%28%29%60%2C+where+%60IO+a%60+is+the+type+of+I%2FO+actions+which+return+a+value+of+type+%60a%60.%0D%0A%2F%2F+%60%28%29%60+is+the+unit+type%2C+which+has+a+unique+value+also+written+as+%60%28%29%60.%0D%0Amain+%3A+IO+%28%29%3B%0D%0Amain+%3D+%28%0D%0A++++%2F%2F+%60println+%3A+String+-%3E+IO+%28%29%60+makes+an+I%2FO+action+that+prints+a+string+%28and+a+newline%29.%0D%0A++++%2F%2F+Roughly+speaking%2C+prefix+operator+%60%2Aact%60+performs+the+I%2FO+action+%60act%60+and+evaluates+to+the+value+returned+by+%60act%60.%0D%0A++++let+_+%3D+%2A%28println+%24+%22truth+%3A+%22+%2B+truth.to_string%29%3B%0D%0A++++let+_+%3D+%2A%28println+%24+%22truth2+%3A+%22+%2B+truth2.to_string%29%3B%0D%0A++++let+_+%3D+%2A%28println+%24+%22calc_fib%2810%29+%3A+%22+%2B+calc_fib%2810%29.to_string%29%3B%0D%0A++++let+_+%3D+%2A%28println+%24+%22calc_fib2%2810%29+%3A+%22+%2B+calc_fib2%2810%29.to_string%29%3B%0D%0A%0D%0A++++%2F%2F+%60pure+%3A+a+-%3E+IO+a%60+creates+an+I%2FO+action+which+does+nothing+and+only+returns+a+value.+%0D%0A++++%2F%2F+By+a+syntax+sugar%2C+you+can+write+%60pure%28%29%60+instead+of+%60pure%28%28%29%29%60.%0D%0A++++pure%28%29%0D%0A%29%3B)
- [Array and loop](https://tttmmmyyyy.github.io/fixlang-playground/index.html?src=module+Main%3B%0D%0A%0D%0A%2F%2F+Prints+30th+value+of+Fibonacci+sequence.%0D%0Amain+%3A+IO+%28%29%3B%0D%0Amain+%3D+%28%0D%0A++++%2F%2F+%60fill+%3A+I64+-%3E+a+-%3E+Array+a%60+in+namespace+%60Std%3A%3AArray%60+creates+an+array+of+specified+length+and+filled+by+a+value.%0D%0A++++let+arr+%3D+Array%3A%3Afill%2831%2C+0%29%3B%0D%0A++++%2F%2F+%60set%60+and+%60set%21%60+of+type+%60I64+-%3E+a+-%3E+Array+a+-%3E+Array+a%60+insert+a+value+into+an+array.%0D%0A++++%2F%2F+%60set%60+updates+the+given+array+in+O%281%29+if+the+reference+counter+of+it+is+one%2C+%0D%0A++++%2F%2F+or+inserts+a+value+after+cloning+the+array+%28it+takes+O%28n%29%29+otherwise.%0D%0A++++%2F%2F+%60set%21%60+always+tries+to+update+the+given+array+in+O%281%29%2C+or+panics+if+the+reference+counter+is+greater+than+one.%0D%0A++++%2F%2F+There+are+also+%60mod%60+and+%60mod%21%60+of+type+%60I64+-%3E+%28a+-%3E+a%29+-%3E+Array+a+-%3E+Array+a%60%2C+which+update+a+value+of+an+array.%0D%0A++++let+arr+%3D+arr.set%21%280%2C+0%29%3B%0D%0A++++let+arr+%3D+arr.set%21%281%2C+1%29%3B%0D%0A++++%2F%2F+A+way+for+loop+is+to+use+%60loop%60%2C+%60continue%60+and+%60break%60.%0D%0A++++%2F%2F+loop+%3A+s+-%3E+LoopResult+s+r+-%3E+r+--+Takes+the+initial+state+and+loop+body%2C+and+performs+loop.%0D%0A++++%2F%2F+continue+%3A+s+-%3E+LoopResult+s+r+--+Takes+the+next+state+and+continues+the+loop.%0D%0A++++%2F%2F+break+%3A+r+-%3E+LoopResult+s+r+--+Breaks+the+loop+and+returns+the+given+value.%0D%0A++++let+arr+%3D+loop%28%282%2C+arr%29%2C+%7C%28idx%2C+arr%29%7C%0D%0A++++++++if+idx+%3D%3D+arr.get_size+%7B%0D%0A++++++++++++break+%24+arr%0D%0A++++++++%7D+else+%7B%0D%0A++++++++++++%2F%2F+To+get+a+value+of+an+array%2C+use+%60%40+%3A+I64+-%3E+Array+a+-%3E+a%60.%0D%0A++++++++++++let+x+%3D+arr.%40%28idx-1%29%3B%0D%0A++++++++++++let+y+%3D+arr.%40%28idx-2%29%3B%0D%0A++++++++++++let+arr+%3D+arr.set%21%28idx%2C+x%2By%29%3B%0D%0A++++++++++++continue+%24+%28idx%2B1%2C+arr%29%0D%0A++++++++%7D%0D%0A++++%29%3B%0D%0A++++println+%24+arr.%40%2830%29.to_string+%2F%2F+832040%0D%0A%29%3B)
- [Structs](https://tttmmmyyyy.github.io/fixlang-playground/index.html?src=module+Main%3B%0D%0A%0D%0A%2F%2F+You+can+define+struct+as+follows%3A%0D%0A%2F%2F+%60F64%60+is+the+type+of+64-bit+floating+values.%0D%0Atype+Quantity+%3D+struct+%7B+value+%3A+F64%2C+unit+%3A+String+%7D%3B%0D%0A%0D%0Anamespace+Quantity+%7B%0D%0A%0D%0A++++make+%3A+F64+-%3E+String+-%3E+Quantity%3B%0D%0A++++make+%3D+%7Cval%2C+unit%7C+%28%0D%0A++++++++%2F%2F+Construction+of+a+struct+value.%0D%0A++++++++Quantity+%7B+value+%3A+val%2C+unit+%3A+unit+%7D%0D%0A++++%29%3B%0D%0A%0D%0A++++stringify+%3A+Quantity+-%3E+String%3B%0D%0A++++stringify+%3D+%7Cq%7C+%28%0D%0A++++++++%2F%2F+To+get+a+field+value%2C+use+the+function+%60%40%28field+name%29+%3A+%28Struct%29+-%3E+%28FieldType%29%60.%0D%0A++++++++q.%40value.to_string+%2B+%22+%22+%2B+q.%40unit%0D%0A++++%29%3B%0D%0A%0D%0A++++add+%3A+Quantity+-%3E+Quantity+-%3E+Quantity%3B%0D%0A++++%2F%2F+Pattern+matching+is+available+in+function+definition.%0D%0A++++add+%3D+%7CQuantity+%7B+value+%3A+lhs_val%2C+unit+%3A+lhs_unit+%7D%2C+rhs%7C+%28%0D%0A++++++++%2F%2F+Pattern+matching+is+also+available+in+let-binding.%0D%0A++++++++let+Quantity+%7B+value+%3A+rhs_val%2C+unit+%3A+rhs_unit+%7D+%3D+rhs%3B%0D%0A++++++++if+lhs_unit+%3D%3D+rhs_unit+%7B%0D%0A++++++++++++Quantity+%7B+value+%3A+lhs_val+%2B+rhs_val%2C+unit+%3A+lhs_unit+%7D%0D%0A++++++++%7D+else+%7B%0D%0A++++++++++++abort%28%29%0D%0A++++++++%7D%0D%0A++++%29%3B%0D%0A%0D%0A++++%2F%2F+%223.0+kg%22+%2A+%222.0+m%22+%3D%3D+%226.0+kg+m%22%0D%0A++++mul+%3A+Quantity+-%3E+Quantity+-%3E+Quantity%3B%0D%0A++++mul+%3D+%7Crhs%2C+lhs%7C+%28+%2F%2F+Note+that+%60lhs.mul%28rhs%29+%3D%3D+mul%28rhs%2C+lhs%29%60%2C+so+we+call+the+first+argument+as+%60rhs%60.%0D%0A++++++++let+val+%3D+lhs.%40value+%2A+rhs.%40value%3B%0D%0A++++++++%2F%2F+%60set_%28field+name%29+%3A+%28FieldType%29+-%3E+%28Struct%29+-%3E+%28Struct%29%60+updates+a+field.%0D%0A++++++++%2F%2F+%60mod_%28field+name%29+%3A+%28%28FieldType%29+-%3E+%28FieldType%29%29+-%3E+%28Struct%29+-%3E+%28Struct%29%60+transforms+a+field.%0D%0A++++++++lhs.set_value%28val%29.mod_unit%28%7Cu%7C+u+%2B+%22+%22+%2B+rhs.%40unit%29%0D%0A++++%29%3B%0D%0A%0D%0A++++%2F%2F+Pair+%28or+tuple%29+is+a+special+struct+with+fields+%600%60+and+%601%60.%0D%0A++++%2F%2F+Field+accessor+functions+%60%400%60%2C+%60%401%60%2C+sette%2Fmodifier+functions+%60set_0%60%2C+%60set_1%60%2C+%60mod_0%60%2C+%60mod_1%60%0D%0A++++%2F%2F+and+pattern+matching+are+available+as+well+as+user-defined+structs.%0D%0A++++from_pair+%3A+%28F64%2C+String%29+-%3E+Quantity%3B%0D%0A++++from_pair+%3D+%7C%28val%2C+unit%29%7C+make%28val%2C+unit%29%3B%0D%0A%7D%0D%0A%0D%0A%2F%2F+You+can+also+define+a+generic+struct+parametrized+by+a+type+variable%3A%0D%0Atype+Quantity2+a+%3D+struct+%7B+value+%3A+a%2C+unit+%3A+String+%7D%3B%0D%0A%0D%0Anamespace+Quantity2+%7B%0D%0A++++make+%3A+a+-%3E+String+-%3E+Quantity2+a%3B%0D%0A++++make+%3D+%7Cval%2C+unit%7C+Quantity2+%7B+value+%3A+val%2C+unit+%3A+unit+%7D%3B%0D%0A%0D%0A++++stringify+%3A+%5Ba+%3A+ToString%5D+Quantity2+a+-%3E+String%3B%0D%0A++++stringify+%3D+%7Cq%7C+q.%40value.to_string+%2B+%22+%22+%2B+q.%40unit%3B%0D%0A%7D%0D%0A%0D%0Amain+%3A+IO+%28%29%3B%0D%0Amain+%3D+%28%0D%0A++++let+x+%3D+Quantity%3A%3Amake%281.0%2C+%22kg%22%29%3B%0D%0A++++let+y+%3D+Quantity%3A%3Amake%282.0%2C+%22kg%22%29%3B%0D%0A++++let+z+%3D+Quantity%3A%3Amake%283.0%2C+%22m%22%29%3B%0D%0A++++let+q+%3D+x.add%28y%29.mul%28z%29%3B+%2F%2F+%281.0+kg+%2B+2.0+kg%29+%2A+3.0+m+%0D%0A++++let+_+%3D+%2A%28println+%24+q.stringify%29%3B%0D%0A++++let+q2+%3D+Quantity2%3A%3Amake%2842%2C+%22s%22%29%3B+%2F%2F+q2+%3A+Quantity2+I64%0D%0A++++let+_+%3D+%2A%28println+%24+q2.stringify%29%3B%0D%0A++++pure%28%29%0D%0A%29%3B%0D%0A)
- [Unions](https://tttmmmyyyy.github.io/fixlang-playground/index.html?src=module+Main%3B%0D%0A%0D%0Aimport+Math%3B+%2F%2F+for+pi64%0D%0A%0D%0A%2F%2F+Tagged+union+can+be+defined+as+follows%3A%0D%0Atype+Angle+%3D+union+%7B+radian%3A+F64%2C+degree%3A+F64+%7D%3B%0D%0A%0D%0A%2F%2F+You+can+define+generic+unions+by+writing+%60type+SomeUnion+a+%3D+union+%7B+...%28use+type+%60a%60+for+type+of+fields%29...+%7D%3B%60%0D%0A%0D%0Anamespace+Angle+%7B%0D%0A++++to_degree+%3A+Angle+-%3E+Angle%3B%0D%0A++++to_degree+%3D+%7Ca%7C+%28%0D%0A++++++++%2F%2F+%60is_%28variant%29+%3A+%28Union%29+-%3E+Bool%60+checks+whether+the+union+value+is+a+specific+variant.%0D%0A++++++++if+a.is_degree+%7B+a+%7D%3B%0D%0A++++++++%2F%2F+%60%28variant%29+%3A+%28VariantType%29+-%3E+%28Union%29%60+constructs+an+union+value.%0D%0A++++++++%2F%2F+%60as_%28variant%29+%3A+%28Union%29+-%3E+%28VariantType%29%60+extracts+a+value+from+an+union+value+%28or+panics%29.%0D%0A++++++++Angle%3A%3Adegree%28a.as_radian+%2A+180.0+%2F+Math%3A%3Api64%29%0D%0A++++%29%3B%0D%0A%0D%0A++++stringify_as_degree+%3A+Angle+-%3E+String%3B%0D%0A++++stringify_as_degree+%3D+%7Ca%7C+%28%0D%0A++++++++let+a+%3D+a.to_degree%3B%0D%0A++++++++a.as_degree.to_string+%2B+%22+deg%22+%0D%0A++++%29%3B%0D%0A%7D%0D%0A%0D%0Amain+%3A+IO+%28%29%3B%0D%0Amain+%3D+%28%0D%0A++++println+%24+Angle%3A%3Aradian%28Math%3A%3Api64+%2F+6.0%29.stringify_as_degree+%2F%2F+30+degree%0D%0A%29%3B%0D%0A)
- [Traits](https://tttmmmyyyy.github.io/fixlang-playground/index.html?src2=bW9kdWxlIE1haW47DQoNCi8vIFlvdSBjYW4gZGVmaW5lIGEgdHJhaXQgYW5kIGltcGxlbWVudCBpdCBhcyBmb2xsb3dzOg0KdHJhaXQgYSA6IFNlbGZJbnRyb2R1Y3Rpb24gew0KICAgIC8vIEFuIElPIGFjdGlvbiB3aGljaCBpbnRyb2R1Y2VzIHRoZSBnaXZlbiB2YWx1ZS4NCiAgICBpbnRyb2R1Y2Vfc2VsZiA6IGEgLT4gSU8gKCk7DQp9DQoNCmltcGwgSTY0IDogU2VsZkludHJvZHVjdGlvbiB7DQogICAgaW50cm9kdWNlX3NlbGYgPSB8bnwgcHJpbnRsbiAkICJIaSEgSSdtIGEgNjQtYml0IGludGVnZXIgIiArIG4udG9fc3RyaW5nICsgIiEiOw0KfQ0KDQovKg0KYEVxYCB0cmFpdCBpcyBkZWZpbmVkIGluIHN0YW5kYXJkIGxpYnJhcnkgYXMgZm9sbG93czogDQoNCmBgYA0KdHJhaXQgYSA6IEVxIHsNCiAgICBlcSA6IGEgLT4gYSAtPiBCb29sDQp9DQpgYGANCg0KRXhwcmVzc2lvbiBgeCA9PSB5YCBpcyBpbnRlcnByZXRlZCBhcyBgRXE6OmVxKHgsIHkpYC4NCiovDQoNCi8vIEFzIGFub3RoZXIgZXhhbXBsZSwgDQp0eXBlIFBhaXIgYSBiID0gc3RydWN0IHsgZnN0OiBhLCBzbmQ6IGIgfTsNCg0KLy8gSW4gdGhlIHRyYWl0IGltcGxlbWVudGF0aW9uLCB5b3UgY2FuIHNwZWNpZnkgcHJlY29uZGl0aW9ucyBvbiB0eXBlIHZhcmlhYmxlcyBpbiBgW11gIGJyYWNrZXQgYWZ0ZXIgYGltcGxgLg0KaW1wbCBbYSA6IEVxLCBiIDogRXFdIFBhaXIgYSBiIDogRXEgew0KICAgIGVxID0gfGxocywgcmhzfCAoDQogICAgICAgIGxocy5AZnN0ID09IHJocy5AZnN0ICYmIGxocy5Ac25kID09IHJocy5Ac25kDQogICAgKTsNCn0NCg0KLy8gWW91IGNhbiBzcGVjaWZ5IHByZWNvbmRpdGlvbnMgb2YgdHlwZSB2YXJpYWJsZXMgaW4gdGhlIGBbXWAgYnJhY2tldCBiZWZvcmUgdHlwZSBzaWduYXR1cmUuDQpzZWFyY2ggOiBbYSA6IEVxXSBhIC0%2BIEFycmF5IGEgLT4gSTY0Ow0Kc2VhcmNoID0gfGVsZW0sIGFycnwgbG9vcCgwLCB8aWR4fA0KICAgIGlmIGlkeCA9PSBhcnIuZ2V0X3NpemUgeyBicmVhayAkIC0xIH07DQogICAgaWYgYXJyLkAoaWR4KSA9PSBlbGVtIHsgYnJlYWsgJCBpZHggfTsNCiAgICBjb250aW51ZSAkIChpZHggKyAxKQ0KKTsNCg0KLy8gQW4gZXhhbXBsZSBvZiBkZWZpbmluZyBoaWdoZXIta2luZGVkIHRyYWl0Lg0KLy8gQWxsIHR5cGUgdmFyaWFibGUgaGFzIGtpbmQgYCpgIGJ5IGRlZmF1bHQsIGFuZCBhbnkga2luZCBvZiBoaWdoZXIta2luZGVkIHR5cGUgdmFyaWFibGUgbmVlZCB0byBiZSBhbm5vdGVkIGV4cGxpY2l0bHkuDQp0cmFpdCBbZiA6ICotPipdIGYgOiBNeUZ1bmN0b3Igew0KICAgIG15bWFwIDogKGEgLT4gYikgLT4gZiBhIC0%2BIGYgYjsNCn0NCg0KLy8gQW4gZXhhbXBsZSBvZiBpbXBsZW1lbnRpbmcgaGlnaGVyLWtpbmRlZCB0cmFpdC4NCi8vIGBBcnJheWAgaXMgYSB0eXBlIG9mIGtpbmQgYCogLT4gKmAsIHNvIG1hdGNoZXMgdG8gdGhlIGtpbmQgb2YgdHJhaXQgYE15RnVuY3RvcmAuDQppbXBsIEFycmF5IDogTXlGdW5jdG9yIHsNCiAgICBteW1hcCA9IHxmLCBhcnJ8ICgNCiAgICAgICAgQXJyYXk6OmZyb21fbWFwKGFyci5nZXRfc2l6ZSwgfGlkeHwgZihhcnIuQChpZHgpKSkNCiAgICApOw0KfQ0KDQptYWluIDogSU8gKCk7DQptYWluID0gKA0KICAgIGxldCBhcnIgPSBBcnJheTo6ZnJvbV9tYXAoNiwgfHh8IHgpOyAvLyBhcnIgPSBbMCwxLDIsLi4uLDldLg0KICAgIGxldCBhcnIgPSBhcnIubXltYXAofHh8IFBhaXIgeyBmc3Q6IHggJSAyLCBzbmQ6IHggJSAzIH0pOyAvLyBhcnIgPSBbKDAsIDApLCAoMSwgMSksICgwLCAyKSwgLi4uXS4NCiAgICBsZXQgeCA9IGFyci5zZWFyY2goUGFpciB7IGZzdDogMSwgc25kOiAyfSk7IC8vIDUsIHRoZSBmaXJzdCBudW1iZXIgeCBzdWNoIHRoYXQgeCAlIDIgPT0gMSBhbmQgeCAlIDMgPT0gMi4NCiAgICB4LmludHJvZHVjZV9zZWxmDQopOw%3D%3D)
- [Iterators](https://tttmmmyyyy.github.io/fixlang-playground/index.html?src2=bW9kdWxlIE1haW47DQoNCmltcG9ydCBNYXRoOyAvLyBmb3IgTWF0aDo6Z2NkDQoNCi8vIEl0ZXJhdG9yLCBhLmsuYS4gbGF6eSBsaXN0LCBpcyBkZWZpbmVkIGFzIGZvbGxvd3MuDQovLyBgdHlwZSBJdGVyYXRvciBhID0gdW5ib3ggc3RydWN0IHsgbmV4dDogKCkgLT4gT3B0aW9uIChhLCBJdGVyYXRvciBhKSB9O2ANCg0KLy8gSW5zdGVhZCBvZiBjb250YWluaW5nICJ0aGUgbmV4dCB2YWx1ZSIsIGEgbm9uLWVtcHR5IGl0ZXJhdG9yIGhhcyBhIGZ1bmN0aW9uIHRvIGdlbmVyYXRlIGEgcGFpciBvZg0KLy8gLSB0aGUgbmV4dCB2YWx1ZSBhbmQgDQovLyAtIHRoZSBpdGVyYXRvciB0byBnZW5lcmF0ZSByZXN0IHZhbHVlcy4NCg0KLy8gSXRlcmF0b3JzIGFyZSBpbXBvcnRhbnQgdG8gd3JpdGUgcHJvZ3JhbSBpbiBhIGZ1bmN0aW9uYWwgbWFubmVyLiANCi8vIFRoZSBmb2xsb3dpbmcgZXhhbXBsZSBpbGx1c3RyYXRlcyB0aGUgcG93ZXIgb2YgaXRlcmF0b3JzLg0KDQovLyBDb3VudCBkaXZpc29ycyBvZiBhIG51bWJlci4NCi8vIEZvciBleGFtcGxlLCBkaXZpc29ycyBvZiAxMDAgYXJlIDEsIDIsIDQsIDUsIDEwLCAyMCwgMjUsIDUwLCAxMDAsIA0KLy8gd2hpY2ggY2FuIGJlIGdyb3VwZWQgaW50byBhcyB7MSwgMTAwfSwgezIsIDUwfSwgezQsIDI1fSwgezEwfS4NCi8vIFNvIGBjb3VudF9kaXZzKDEwMCkgPT0gMiArIDIgKyAyICsgMiArIDFgLiANCmNvdW50X2RpdnMgOiBJNjQgLT4gSTY0Ow0KY291bnRfZGl2cyA9IHxufCAoDQogICAgSXRlcmF0b3I6OmNvdW50X3VwKDEpIC8vIEdlbmVyYXRlIGFuIGluZmluaXRlIGl0ZXJhdG9yIGAxLCAyLCAzLCAuLi5gIHdoaWNoIGFyZSBjYW5kaWRhdGVzIGZvciBkaXZpc29ycyBvZiBgbmAuDQogICAgICAgIC50YWtlX3doaWxlKHxkfCBkKmQgPD0gbikgLy8gVGFrZSBlbGVtZW50cyBsZXNzIHRoYW4gb3IgZXF1YWwgdG8gcm9vdCBvZiBgbmAuDQogICAgICAgIC5maWx0ZXIofGR8IG4lZCA9PSAwKSAvLyBUYWtlIG9ubHkgZGl2aXNvcnMuDQogICAgICAgIC5tYXAofGR8IGlmIGQqZCA9PSBuIHsgMSB9IGVsc2UgeyAyIH0pIC8vIENvbnZlcnQgYSBkaXZpc29yIGludG8gdGhlIHNpemUgb2YgdGhlIGdyb3VwIGl0IGJlbG9uZ3MgdG8uDQogICAgICAgIC5mb2xkKDAsIEFkZDo6YWRkKSAvLyBTdW0gdXAgdGhlIGl0ZXJhdG9yLiBgZm9sZGAgZm9sZHMgYW4gaXRlcmF0b3IgYnkgYW4gb3BlcmF0b3IgKHR3by12YXJpYWJsZSBmdW5jdGlvbikuIA0KICAgICAgICAvLyBgQWRkOjphZGQgOiBJNjQgLT4gSTY0IC0%2BIEk2NGAgYWRkcyB0d28gaW50ZWdlcnMuDQopOw0KDQovLyBJbmZpbml0ZSBpdGVyYXRvciBvZiBwb3NpdGl2ZSByYXRpb25hbCBudW1iZXJzIGxlc3MgdGhhbiAxLg0KcmF0aW9uYWxzIDogSXRlcmF0b3IgKEk2NCwgSTY0KTsgLy8gUGFpciBvZiBudW1lcmF0b3IgYW5kIGRlbm9taW5hdG9yLg0KcmF0aW9uYWxzID0gKA0KICAgIEl0ZXJhdG9yOjpjb3VudF91cCgxKSAvLyBJdGVyYXRvciBvZiBkZW5vbWluYXRvcnMNCiAgICAgICAgLm1hcCh8ZHwgKA0KICAgICAgICAgICAgSXRlcmF0b3I6OnJhbmdlKDEsIGQpIC8vIEl0ZXJhdG9yIG9mIG51bWVyYXRvcnMNCiAgICAgICAgICAgICAgICAuZmlsdGVyKHxufCBnY2QobiwgZCkgPT0gMSkgLy8gRmlsdGVyIG91dCBudW1lcmF0b3JzIHdoaWNoIGhhcyBjb21tb24gZmFjdG9yIHdpdGggdGhlIGRlbm9taW5hdG9yLg0KICAgICAgICAgICAgICAgIC5tYXAofG58IChuLCBkKSkgLy8gTWFrZSBwYWlyIG9mIG51bWVyYXRvciBhbmQgZGVub21pbmF0b3IuDQogICAgICAgICkpDQogICAgICAgIC5mbGF0dGVuIC8vIGBmbGF0dGVuIDogSXRlcmF0b3IgKEl0ZXJhdG9yIGEpIC0%2BIEl0ZXJhdG9yIGFgDQopOw0KDQpzdHJpbmdpZnlfcmF0aW9uYWwgOiAoSTY0LCBJNjQpIC0%2BIFN0cmluZzsNCnN0cmluZ2lmeV9yYXRpb25hbCA9IHwobiwgZCl8IG4udG9fc3RyaW5nICsgIi8iICsgZC50b19zdHJpbmc7DQoNCm1haW4gOiBJTyAoKTsNCm1haW4gPSAoDQogICAgbGV0IF8gPSAqKHByaW50bG4gJCAiTnVtYmVyIG9mIGRpdmlzb3JzIG9mIDEwMCBpcyAiICsgY291bnRfZGl2cygxMDApLnRvX3N0cmluZyArICIuIik7DQogICAgbGV0IF8gPSAqKHByaW50bG4gJCAiRmlyc3QgMTAwIHJhdGlvbmFscyA6ICIgKyByYXRpb25hbHMudGFrZSgxMDApLm1hcChzdHJpbmdpZnlfcmF0aW9uYWwpLmpvaW4oIiwgIikpOw0KICAgIHB1cmUoKQ0KKTsNCg0K)

## Install (macOS / WSL)

- Install [Rust](https://www.rust-lang.org/tools/install).
- Install llvm12.0.1. It is recommended to use [llvmemv](https://crates.io/crates/llvmenv).
    - In macOS, llvmenv installs llvm to "~/Library/Application Support/llvmenv/12.0.1", but llvm-sys currently doesn't understand path with a whitespace correctly, so you need to copy/move "12.0.1" directory to another path.
- Set LLVM_SYS_120_PREFIX variable to the directory to which llvm installed.
- `git clone https://github.com/tttmmmyyyy/fixlang.git && cd fixlang`.
- `cargo install --path .`. Then the compiler command `fix` will be installed to `~/.cargo/bin`.

## Usage

- You can run the source file (with extension ".fix") by `fix run -f {source-file}`.
- If you want to build executable binary, run `fix build -f {source-file}.`.

## Tutorial / references

See [document](/Document.md).

## Discord

https://discord.gg/ad4GakEA7R