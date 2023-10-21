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

- [Basic syntax](https://tttmmmyyyy.github.io/fixlang-playground/index.html?src2=Ly8gRWFjaCBzb3VyY2UgZmlsZSBoYXMgdG8gc3RhcnQgd2l0aCBtb2R1bGUgZGVjbGFyYXRpb24uDQptb2R1bGUgTWFpbjsNCg0KLy8gRGVjbGFyYXRpb24gYW5kIGRlZmluaXRpb24gb2YgZ2xvYmFsIHZhbHVlLg0KLy8gYEk2NGAgaXMgdGhlIHR5cGUgb2YgNjQtYml0IGludGVnZXJzLg0KdHJ1dGggOiBJNjQ7DQp0cnV0aCA9IDQyOyANCg0KLy8gRGVjbGFyYXRpb24gYW5kIGRlZmluaXRpb24gb2YgZ2xvYmFsIChyZWN1cnNpdmUpIGZ1bmN0aW9uLg0KLy8gVG8gZGVmaW5lIGZ1bmN0aW9uLCB3cml0ZSBgfGFyZzAsIGFyZzEsIC4uLnwgKGZ1bmN0aW9uIGJvZHkpYC4NCi8vIChQYXJlbnRoZXNlcyBhcm91bmQgYChmdW5jdGlvbiBib2R5KWAgaXMgbm90IG1hbmRhdG9yeS4pDQovLyBOb3RlIHRoYXQgRml4IGlzIGFuIGV4cHJlc3Npb24gYmFzZWQgbGFuZ3VhZ2UuIFlvdSBkb24ndCBuZWVkIHRvIHdyaXRlICJyZXR1cm4gc3RhdGVtZW50Ii4NCmNhbGNfZmliIDogSTY0IC0%2BIEk2NDsNCmNhbGNfZmliID0gfG58ICgNCiAgICBpZiBuIDw9IDEgeyBuIH0gZWxzZSB7IGNhbGNfZmliKG4tMSkgKyBjYWxjX2ZpYihuLTIpIH0NCik7DQoNCmNhbGNfZmliMiA6IEk2NCAtPiBJNjQ7DQpjYWxjX2ZpYjIgPSB8bnwgKA0KICAgIC8vIEFub3RoZXIgc3ludGF4IG9mIGBpZmAsIGBpZiAoY29uZCkgeyAodGhlbiBleHByKSB9OyAoZWxzZSBleHByKWAsIGNhbiBiZSB1c2VkIHRvIHdyaXRlIGVhcmx5IHJldHVybi4NCiAgICBpZiBuIDw9IDEgeyBuIH07DQoNCiAgICAvLyBVc2UgYGxldGAgdG8gZGVmaW5lIGEgbG9jYWwgbmFtZS4NCiAgICBsZXQgeCA9IGNhbGNfZmliMihuLTEpOw0KICAgIGxldCB5ID0gY2FsY19maWIyKG4tMik7DQogICAgeCArIHkNCik7DQoNCnRydXRoMiA6IEk2NDsNCnRydXRoMiA9ICgNCiAgICAvLyBZb3UgY2FuIGRlZmluZSBsb2NhbCBmdW5jdGlvbiAoY2xvc3VyZSkgbGlrZSB0aGlzLiBgZmAgaGFzIHR5cGUgYEk2NCAtPiBJNjQgLT4gSTY0IC0%2BIEk2NGAuDQogICAgbGV0IGYgPSB8YSwgYiwgY3wgKGEgKyBiKSAqIGM7DQoNCiAgICAvLyBQYXJ0aWFsIGFwcGxpY2F0aW9uLiBgZG91YmxlYCBoYXMgdHlwZSBgSTY0IC0%2BIEk2NGAgYW5kIG1hcHMgYGNgIHRvIGAoMSArIDEpICogYyA9PSAyICogY2AuDQogICAgbGV0IGRvdWJsZSA9IGYoMSwgMSk7DQoNCiAgICAvLyBSaWdodC1hc3NvY2lhdGl2ZSBvcGVyYXRvciBgJGAgYXBwbGllcyBhIGZ1bmN0aW9uIHRvIGEgdmFsdWU6IGBmICQgeCA9PSBmKHgpYCBhbmQgYGYgJCBnICQgeCA9PSBmKGcoeCkpYC4NCiAgICBsZXQgdHdlbHZlID0gZG91YmxlICQgZG91YmxlICQgMzsNCg0KICAgIC8vIGAuYCBpcyBhbm90aGVyIG9wZXJhdG9yIHRvIGFwcGx5IGEgZnVuY3Rpb246IGB4LmYgPT0gZih4KWAuDQogICAgLy8gSXQgaGFzIGxvd2VyIHByaW9yaXR5IHRoYW4gdXN1YWwgZnVuY3Rpb24gY2FsbCwgc28gYDMuZigxLCAyKSA9PSBmKDEsIDIpKDMpID09IGYoMSwgMiwgMylgLg0KICAgIGxldCBuaW5lID0gMy5mKDEsIDIpOw0KDQogICAgZG91YmxlICQgbmluZSArIHR3ZWx2ZQ0KKTsNCg0KLy8gRml4IHByb2dyYW0gY2FsbHMgYE1haW46Om1haW5gIChpLmUuLCBgbWFpbmAgb2YgYE1haW5gIG1vZHVsZSkgYXMgdGhlIGVudHJ5IHBvaW50Lg0KLy8gYE1haW46Om1haW5gIG11c3QgaGF2ZSB0eXBlIGBJTyAoKWAsIHdoZXJlIGBJTyBhYCBpcyB0aGUgdHlwZSBvZiBJL08gYWN0aW9ucyB3aGljaCByZXR1cm4gYSB2YWx1ZSBvZiB0eXBlIGBhYC4NCi8vIGAoKWAgaXMgdGhlIHVuaXQgdHlwZSwgd2hpY2ggaGFzIGEgdW5pcXVlIHZhbHVlIGFsc28gd3JpdHRlbiBhcyBgKClgLg0KbWFpbiA6IElPICgpOw0KbWFpbiA9ICgNCiAgICAvLyBgcHJpbnRsbiA6IFN0cmluZyAtPiBJTyAoKWAgbWFrZXMgYW4gSS9PIGFjdGlvbiB0aGF0IHByaW50cyBhIHN0cmluZyAoYW5kIGEgbmV3bGluZSkuDQogICAgLy8gUm91Z2hseSBzcGVha2luZywgcHJlZml4IG9wZXJhdG9yIGAqYWN0YCBwZXJmb3JtcyB0aGUgSS9PIGFjdGlvbiBgYWN0YCBhbmQgZXZhbHVhdGVzIHRvIHRoZSB2YWx1ZSByZXR1cm5lZCBieSBgYWN0YC4NCiAgICBldmFsICoocHJpbnRsbiAkICJ0cnV0aCA6ICIgKyB0cnV0aC50b19zdHJpbmcpOw0KICAgIGV2YWwgKihwcmludGxuICQgInRydXRoMiA6ICIgKyB0cnV0aDIudG9fc3RyaW5nKTsNCiAgICBldmFsICoocHJpbnRsbiAkICJjYWxjX2ZpYigxMCkgOiAiICsgY2FsY19maWIoMTApLnRvX3N0cmluZyk7DQogICAgZXZhbCAqKHByaW50bG4gJCAiY2FsY19maWIyKDEwKSA6ICIgKyBjYWxjX2ZpYjIoMTApLnRvX3N0cmluZyk7DQoNCiAgICAvLyBgcHVyZSA6IGEgLT4gSU8gYWAgY3JlYXRlcyBhbiBJL08gYWN0aW9uIHdoaWNoIGRvZXMgbm90aGluZyBhbmQgb25seSByZXR1cm5zIGEgdmFsdWUuIA0KICAgIC8vIEJ5IGEgc3ludGF4IHN1Z2FyLCB5b3UgY2FuIHdyaXRlIGBwdXJlKClgIGluc3RlYWQgb2YgYHB1cmUoKCkpYC4NCiAgICBwdXJlKCkNCik7)
- [Array and loop](https://tttmmmyyyy.github.io/fixlang-playground/index.html?src=module+Main%3B%0D%0A%0D%0A%2F%2F+Prints+30th+value+of+Fibonacci+sequence.%0D%0Amain+%3A+IO+%28%29%3B%0D%0Amain+%3D+%28%0D%0A++++%2F%2F+%60fill+%3A+I64+-%3E+a+-%3E+Array+a%60+in+namespace+%60Std%3A%3AArray%60+creates+an+array+of+specified+length+and+filled+by+a+value.%0D%0A++++let+arr+%3D+Array%3A%3Afill%2831%2C+0%29%3B%0D%0A++++%2F%2F+%60set%60+and+%60set%21%60+of+type+%60I64+-%3E+a+-%3E+Array+a+-%3E+Array+a%60+insert+a+value+into+an+array.%0D%0A++++%2F%2F+%60set%60+updates+the+given+array+in+O%281%29+if+the+reference+counter+of+it+is+one%2C+%0D%0A++++%2F%2F+or+inserts+a+value+after+cloning+the+array+%28it+takes+O%28n%29%29+otherwise.%0D%0A++++%2F%2F+%60set%21%60+always+tries+to+update+the+given+array+in+O%281%29%2C+or+panics+if+the+reference+counter+is+greater+than+one.%0D%0A++++%2F%2F+There+are+also+%60mod%60+and+%60mod%21%60+of+type+%60I64+-%3E+%28a+-%3E+a%29+-%3E+Array+a+-%3E+Array+a%60%2C+which+update+a+value+of+an+array.%0D%0A++++let+arr+%3D+arr.set%21%280%2C+0%29%3B%0D%0A++++let+arr+%3D+arr.set%21%281%2C+1%29%3B%0D%0A++++%2F%2F+A+way+for+loop+is+to+use+%60loop%60%2C+%60continue%60+and+%60break%60.%0D%0A++++%2F%2F+loop+%3A+s+-%3E+LoopResult+s+r+-%3E+r+--+Takes+the+initial+state+and+loop+body%2C+and+performs+loop.%0D%0A++++%2F%2F+continue+%3A+s+-%3E+LoopResult+s+r+--+Takes+the+next+state+and+continues+the+loop.%0D%0A++++%2F%2F+break+%3A+r+-%3E+LoopResult+s+r+--+Breaks+the+loop+and+returns+the+given+value.%0D%0A++++let+arr+%3D+loop%28%282%2C+arr%29%2C+%7C%28idx%2C+arr%29%7C%0D%0A++++++++if+idx+%3D%3D+arr.get_size+%7B%0D%0A++++++++++++break+%24+arr%0D%0A++++++++%7D+else+%7B%0D%0A++++++++++++%2F%2F+To+get+a+value+of+an+array%2C+use+%60%40+%3A+I64+-%3E+Array+a+-%3E+a%60.%0D%0A++++++++++++let+x+%3D+arr.%40%28idx-1%29%3B%0D%0A++++++++++++let+y+%3D+arr.%40%28idx-2%29%3B%0D%0A++++++++++++let+arr+%3D+arr.set%21%28idx%2C+x%2By%29%3B%0D%0A++++++++++++continue+%24+%28idx%2B1%2C+arr%29%0D%0A++++++++%7D%0D%0A++++%29%3B%0D%0A++++println+%24+arr.%40%2830%29.to_string+%2F%2F+832040%0D%0A%29%3B)
- [Structs](https://tttmmmyyyy.github.io/fixlang-playground/index.html?src2=bW9kdWxlIE1haW47DQoNCi8vIFlvdSBjYW4gZGVmaW5lIHN0cnVjdCBhcyBmb2xsb3dzOg0KLy8gYEY2NGAgaXMgdGhlIHR5cGUgb2YgNjQtYml0IGZsb2F0aW5nIHZhbHVlcy4NCnR5cGUgUXVhbnRpdHkgPSBzdHJ1Y3QgeyB2YWx1ZSA6IEY2NCwgdW5pdCA6IFN0cmluZyB9Ow0KDQpuYW1lc3BhY2UgUXVhbnRpdHkgew0KDQogICAgbWFrZSA6IEY2NCAtPiBTdHJpbmcgLT4gUXVhbnRpdHk7DQogICAgbWFrZSA9IHx2YWwsIHVuaXR8ICgNCiAgICAgICAgLy8gQ29uc3RydWN0aW9uIG9mIGEgc3RydWN0IHZhbHVlLg0KICAgICAgICBRdWFudGl0eSB7IHZhbHVlIDogdmFsLCB1bml0IDogdW5pdCB9DQogICAgKTsNCg0KICAgIHN0cmluZ2lmeSA6IFF1YW50aXR5IC0%2BIFN0cmluZzsNCiAgICBzdHJpbmdpZnkgPSB8cXwgKA0KICAgICAgICAvLyBUbyBnZXQgYSBmaWVsZCB2YWx1ZSwgdXNlIHRoZSBmdW5jdGlvbiBgQChmaWVsZCBuYW1lKSA6IChTdHJ1Y3QpIC0%2BIChGaWVsZFR5cGUpYC4NCiAgICAgICAgcS5AdmFsdWUudG9fc3RyaW5nICsgIiAiICsgcS5AdW5pdA0KICAgICk7DQoNCiAgICBhZGQgOiBRdWFudGl0eSAtPiBRdWFudGl0eSAtPiBRdWFudGl0eTsNCiAgICAvLyBQYXR0ZXJuIG1hdGNoaW5nIGlzIGF2YWlsYWJsZSBpbiBmdW5jdGlvbiBkZWZpbml0aW9uLg0KICAgIGFkZCA9IHxRdWFudGl0eSB7IHZhbHVlIDogbGhzX3ZhbCwgdW5pdCA6IGxoc191bml0IH0sIHJoc3wgKA0KICAgICAgICAvLyBQYXR0ZXJuIG1hdGNoaW5nIGlzIGFsc28gYXZhaWxhYmxlIGluIGxldC1iaW5kaW5nLg0KICAgICAgICBsZXQgUXVhbnRpdHkgeyB2YWx1ZSA6IHJoc192YWwsIHVuaXQgOiByaHNfdW5pdCB9ID0gcmhzOw0KICAgICAgICBpZiBsaHNfdW5pdCA9PSByaHNfdW5pdCB7DQogICAgICAgICAgICBRdWFudGl0eSB7IHZhbHVlIDogbGhzX3ZhbCArIHJoc192YWwsIHVuaXQgOiBsaHNfdW5pdCB9DQogICAgICAgIH0gZWxzZSB7DQogICAgICAgICAgICBhYm9ydCgpDQogICAgICAgIH0NCiAgICApOw0KDQogICAgLy8gIjMuMCBrZyIgKiAiMi4wIG0iID09ICI2LjAga2cgbSINCiAgICBtdWwgOiBRdWFudGl0eSAtPiBRdWFudGl0eSAtPiBRdWFudGl0eTsNCiAgICBtdWwgPSB8cmhzLCBsaHN8ICggLy8gTm90ZSB0aGF0IGBsaHMubXVsKHJocykgPT0gbXVsKHJocywgbGhzKWAsIHNvIHdlIGNhbGwgdGhlIGZpcnN0IGFyZ3VtZW50IGFzIGByaHNgLg0KICAgICAgICBsZXQgdmFsID0gbGhzLkB2YWx1ZSAqIHJocy5AdmFsdWU7DQogICAgICAgIC8vIGBzZXRfKGZpZWxkIG5hbWUpIDogKEZpZWxkVHlwZSkgLT4gKFN0cnVjdCkgLT4gKFN0cnVjdClgIHVwZGF0ZXMgYSBmaWVsZC4NCiAgICAgICAgLy8gYG1vZF8oZmllbGQgbmFtZSkgOiAoKEZpZWxkVHlwZSkgLT4gKEZpZWxkVHlwZSkpIC0%2BIChTdHJ1Y3QpIC0%2BIChTdHJ1Y3QpYCB0cmFuc2Zvcm1zIGEgZmllbGQuDQogICAgICAgIGxocy5zZXRfdmFsdWUodmFsKS5tb2RfdW5pdCh8dXwgdSArICIgIiArIHJocy5AdW5pdCkNCiAgICApOw0KDQogICAgLy8gUGFpciAob3IgdHVwbGUpIGlzIGEgc3BlY2lhbCBzdHJ1Y3Qgd2l0aCBmaWVsZHMgYDBgIGFuZCBgMWAuDQogICAgLy8gRmllbGQgYWNjZXNzb3IgZnVuY3Rpb25zIGBAMGAsIGBAMWAsIHNldHRlL21vZGlmaWVyIGZ1bmN0aW9ucyBgc2V0XzBgLCBgc2V0XzFgLCBgbW9kXzBgLCBgbW9kXzFgDQogICAgLy8gYW5kIHBhdHRlcm4gbWF0Y2hpbmcgYXJlIGF2YWlsYWJsZSBhcyB3ZWxsIGFzIHVzZXItZGVmaW5lZCBzdHJ1Y3RzLg0KICAgIGZyb21fcGFpciA6IChGNjQsIFN0cmluZykgLT4gUXVhbnRpdHk7DQogICAgZnJvbV9wYWlyID0gfCh2YWwsIHVuaXQpfCBtYWtlKHZhbCwgdW5pdCk7DQp9DQoNCi8vIFlvdSBjYW4gYWxzbyBkZWZpbmUgYSBnZW5lcmljIHN0cnVjdCBwYXJhbWV0cml6ZWQgYnkgYSB0eXBlIHZhcmlhYmxlOg0KdHlwZSBRdWFudGl0eTIgYSA9IHN0cnVjdCB7IHZhbHVlIDogYSwgdW5pdCA6IFN0cmluZyB9Ow0KDQpuYW1lc3BhY2UgUXVhbnRpdHkyIHsNCiAgICBtYWtlIDogYSAtPiBTdHJpbmcgLT4gUXVhbnRpdHkyIGE7DQogICAgbWFrZSA9IHx2YWwsIHVuaXR8IFF1YW50aXR5MiB7IHZhbHVlIDogdmFsLCB1bml0IDogdW5pdCB9Ow0KDQogICAgc3RyaW5naWZ5IDogW2EgOiBUb1N0cmluZ10gUXVhbnRpdHkyIGEgLT4gU3RyaW5nOw0KICAgIHN0cmluZ2lmeSA9IHxxfCBxLkB2YWx1ZS50b19zdHJpbmcgKyAiICIgKyBxLkB1bml0Ow0KfQ0KDQptYWluIDogSU8gKCk7DQptYWluID0gKA0KICAgIGxldCB4ID0gUXVhbnRpdHk6Om1ha2UoMS4wLCAia2ciKTsNCiAgICBsZXQgeSA9IFF1YW50aXR5OjptYWtlKDIuMCwgImtnIik7DQogICAgbGV0IHogPSBRdWFudGl0eTo6bWFrZSgzLjAsICJtIik7DQogICAgbGV0IHEgPSB4LmFkZCh5KS5tdWwoeik7IC8vICgxLjAga2cgKyAyLjAga2cpICogMy4wIG0gDQogICAgZXZhbCAqKHByaW50bG4gJCBxLnN0cmluZ2lmeSk7DQogICAgbGV0IHEyID0gUXVhbnRpdHkyOjptYWtlKDQyLCAicyIpOyAvLyBxMiA6IFF1YW50aXR5MiBJNjQNCiAgICBldmFsICoocHJpbnRsbiAkIHEyLnN0cmluZ2lmeSk7DQogICAgcHVyZSgpDQopOw0K)
- [Unions](https://tttmmmyyyy.github.io/fixlang-playground/index.html?src=module+Main%3B%0D%0A%0D%0Aimport+Math%3B+%2F%2F+for+pi64%0D%0A%0D%0A%2F%2F+Tagged+union+can+be+defined+as+follows%3A%0D%0Atype+Angle+%3D+union+%7B+radian%3A+F64%2C+degree%3A+F64+%7D%3B%0D%0A%0D%0A%2F%2F+You+can+define+generic+unions+by+writing+%60type+SomeUnion+a+%3D+union+%7B+...%28use+type+%60a%60+for+type+of+fields%29...+%7D%3B%60%0D%0A%0D%0Anamespace+Angle+%7B%0D%0A++++to_degree+%3A+Angle+-%3E+Angle%3B%0D%0A++++to_degree+%3D+%7Ca%7C+%28%0D%0A++++++++%2F%2F+%60is_%28variant%29+%3A+%28Union%29+-%3E+Bool%60+checks+whether+the+union+value+is+a+specific+variant.%0D%0A++++++++if+a.is_degree+%7B+a+%7D%3B%0D%0A++++++++%2F%2F+%60%28variant%29+%3A+%28VariantType%29+-%3E+%28Union%29%60+constructs+an+union+value.%0D%0A++++++++%2F%2F+%60as_%28variant%29+%3A+%28Union%29+-%3E+%28VariantType%29%60+extracts+a+value+from+an+union+value+%28or+panics%29.%0D%0A++++++++Angle%3A%3Adegree%28a.as_radian+%2A+180.0+%2F+Math%3A%3Api64%29%0D%0A++++%29%3B%0D%0A%0D%0A++++stringify_as_degree+%3A+Angle+-%3E+String%3B%0D%0A++++stringify_as_degree+%3D+%7Ca%7C+%28%0D%0A++++++++let+a+%3D+a.to_degree%3B%0D%0A++++++++a.as_degree.to_string+%2B+%22+deg%22+%0D%0A++++%29%3B%0D%0A%7D%0D%0A%0D%0Amain+%3A+IO+%28%29%3B%0D%0Amain+%3D+%28%0D%0A++++println+%24+Angle%3A%3Aradian%28Math%3A%3Api64+%2F+6.0%29.stringify_as_degree+%2F%2F+30+degree%0D%0A%29%3B%0D%0A)
- [Traits](https://tttmmmyyyy.github.io/fixlang-playground/index.html?src2=bW9kdWxlIE1haW47DQoNCi8vIFlvdSBjYW4gZGVmaW5lIGEgdHJhaXQgYW5kIGltcGxlbWVudCBpdCBhcyBmb2xsb3dzOg0KdHJhaXQgYSA6IFNlbGZJbnRyb2R1Y3Rpb24gew0KICAgIC8vIEFuIElPIGFjdGlvbiB3aGljaCBpbnRyb2R1Y2VzIHRoZSBnaXZlbiB2YWx1ZS4NCiAgICBpbnRyb2R1Y2Vfc2VsZiA6IGEgLT4gSU8gKCk7DQp9DQoNCmltcGwgSTY0IDogU2VsZkludHJvZHVjdGlvbiB7DQogICAgaW50cm9kdWNlX3NlbGYgPSB8bnwgcHJpbnRsbiAkICJIaSEgSSdtIGEgNjQtYml0IGludGVnZXIgIiArIG4udG9fc3RyaW5nICsgIiEiOw0KfQ0KDQovKg0KYEVxYCB0cmFpdCBpcyBkZWZpbmVkIGluIHN0YW5kYXJkIGxpYnJhcnkgYXMgZm9sbG93czogDQoNCmBgYA0KdHJhaXQgYSA6IEVxIHsNCiAgICBlcSA6IGEgLT4gYSAtPiBCb29sDQp9DQpgYGANCg0KRXhwcmVzc2lvbiBgeCA9PSB5YCBpcyBpbnRlcnByZXRlZCBhcyBgRXE6OmVxKHgsIHkpYC4NCiovDQoNCi8vIEFzIGFub3RoZXIgZXhhbXBsZSwgDQp0eXBlIFBhaXIgYSBiID0gc3RydWN0IHsgZnN0OiBhLCBzbmQ6IGIgfTsNCg0KLy8gSW4gdGhlIHRyYWl0IGltcGxlbWVudGF0aW9uLCB5b3UgY2FuIHNwZWNpZnkgcHJlY29uZGl0aW9ucyBvbiB0eXBlIHZhcmlhYmxlcyBpbiBgW11gIGJyYWNrZXQgYWZ0ZXIgYGltcGxgLg0KaW1wbCBbYSA6IEVxLCBiIDogRXFdIFBhaXIgYSBiIDogRXEgew0KICAgIGVxID0gfGxocywgcmhzfCAoDQogICAgICAgIGxocy5AZnN0ID09IHJocy5AZnN0ICYmIGxocy5Ac25kID09IHJocy5Ac25kDQogICAgKTsNCn0NCg0KLy8gWW91IGNhbiBzcGVjaWZ5IHByZWNvbmRpdGlvbnMgb2YgdHlwZSB2YXJpYWJsZXMgaW4gdGhlIGBbXWAgYnJhY2tldCBiZWZvcmUgdHlwZSBzaWduYXR1cmUuDQpzZWFyY2ggOiBbYSA6IEVxXSBhIC0%2BIEFycmF5IGEgLT4gSTY0Ow0Kc2VhcmNoID0gfGVsZW0sIGFycnwgbG9vcCgwLCB8aWR4fA0KICAgIGlmIGlkeCA9PSBhcnIuZ2V0X3NpemUgeyBicmVhayAkIC0xIH07DQogICAgaWYgYXJyLkAoaWR4KSA9PSBlbGVtIHsgYnJlYWsgJCBpZHggfTsNCiAgICBjb250aW51ZSAkIChpZHggKyAxKQ0KKTsNCg0KLy8gQW4gZXhhbXBsZSBvZiBkZWZpbmluZyBoaWdoZXIta2luZGVkIHRyYWl0Lg0KLy8gQWxsIHR5cGUgdmFyaWFibGUgaGFzIGtpbmQgYCpgIGJ5IGRlZmF1bHQsIGFuZCBhbnkga2luZCBvZiBoaWdoZXIta2luZGVkIHR5cGUgdmFyaWFibGUgbmVlZCB0byBiZSBhbm5vdGVkIGV4cGxpY2l0bHkuDQp0cmFpdCBbZiA6ICotPipdIGYgOiBNeUZ1bmN0b3Igew0KICAgIG15bWFwIDogKGEgLT4gYikgLT4gZiBhIC0%2BIGYgYjsNCn0NCg0KLy8gQW4gZXhhbXBsZSBvZiBpbXBsZW1lbnRpbmcgaGlnaGVyLWtpbmRlZCB0cmFpdC4NCi8vIGBBcnJheWAgaXMgYSB0eXBlIG9mIGtpbmQgYCogLT4gKmAsIHNvIG1hdGNoZXMgdG8gdGhlIGtpbmQgb2YgdHJhaXQgYE15RnVuY3RvcmAuDQppbXBsIEFycmF5IDogTXlGdW5jdG9yIHsNCiAgICBteW1hcCA9IHxmLCBhcnJ8ICgNCiAgICAgICAgQXJyYXk6OmZyb21fbWFwKGFyci5nZXRfc2l6ZSwgfGlkeHwgZihhcnIuQChpZHgpKSkNCiAgICApOw0KfQ0KDQptYWluIDogSU8gKCk7DQptYWluID0gKA0KICAgIGxldCBhcnIgPSBBcnJheTo6ZnJvbV9tYXAoNiwgfHh8IHgpOyAvLyBhcnIgPSBbMCwxLDIsLi4uLDldLg0KICAgIGxldCBhcnIgPSBhcnIubXltYXAofHh8IFBhaXIgeyBmc3Q6IHggJSAyLCBzbmQ6IHggJSAzIH0pOyAvLyBhcnIgPSBbKDAsIDApLCAoMSwgMSksICgwLCAyKSwgLi4uXS4NCiAgICBsZXQgeCA9IGFyci5zZWFyY2goUGFpciB7IGZzdDogMSwgc25kOiAyfSk7IC8vIDUsIHRoZSBmaXJzdCBudW1iZXIgeCBzdWNoIHRoYXQgeCAlIDIgPT0gMSBhbmQgeCAlIDMgPT0gMi4NCiAgICB4LmludHJvZHVjZV9zZWxmDQopOw%3D%3D)
- [Iterators](https://tttmmmyyyy.github.io/fixlang-playground/index.html?src2=bW9kdWxlIE1haW47DQoNCmltcG9ydCBNYXRoOyAvLyBmb3IgTWF0aDo6Z2NkDQoNCi8vIEl0ZXJhdG9yLCBhLmsuYS4gbGF6eSBsaXN0LCBpcyBkZWZpbmVkIGFzIGZvbGxvd3MuDQovLyBgdHlwZSBJdGVyYXRvciBhID0gdW5ib3ggc3RydWN0IHsgbmV4dDogKCkgLT4gT3B0aW9uIChhLCBJdGVyYXRvciBhKSB9O2ANCg0KLy8gSW5zdGVhZCBvZiBjb250YWluaW5nICJ0aGUgbmV4dCB2YWx1ZSIsIGEgbm9uLWVtcHR5IGl0ZXJhdG9yIGhhcyBhIGZ1bmN0aW9uIHRvIGdlbmVyYXRlIGEgcGFpciBvZg0KLy8gLSB0aGUgbmV4dCB2YWx1ZSBhbmQgDQovLyAtIHRoZSBpdGVyYXRvciB0byBnZW5lcmF0ZSByZXN0IHZhbHVlcy4NCg0KLy8gSXRlcmF0b3JzIGFyZSBpbXBvcnRhbnQgdG8gd3JpdGUgcHJvZ3JhbSBpbiBhIGZ1bmN0aW9uYWwgbWFubmVyLiANCi8vIFRoZSBmb2xsb3dpbmcgZXhhbXBsZSBpbGx1c3RyYXRlcyB0aGUgcG93ZXIgb2YgaXRlcmF0b3JzLg0KDQovLyBDb3VudCBkaXZpc29ycyBvZiBhIG51bWJlci4NCi8vIEZvciBleGFtcGxlLCBkaXZpc29ycyBvZiAxMDAgYXJlIDEsIDIsIDQsIDUsIDEwLCAyMCwgMjUsIDUwLCAxMDAsIA0KLy8gd2hpY2ggY2FuIGJlIGdyb3VwZWQgaW50byBhcyB7MSwgMTAwfSwgezIsIDUwfSwgezQsIDI1fSwgezEwfS4NCi8vIFNvIGBjb3VudF9kaXZzKDEwMCkgPT0gMiArIDIgKyAyICsgMiArIDFgLiANCmNvdW50X2RpdnMgOiBJNjQgLT4gSTY0Ow0KY291bnRfZGl2cyA9IHxufCAoDQogICAgSXRlcmF0b3I6OmNvdW50X3VwKDEpIC8vIEdlbmVyYXRlIGFuIGluZmluaXRlIGl0ZXJhdG9yIGAxLCAyLCAzLCAuLi5gIHdoaWNoIGFyZSBjYW5kaWRhdGVzIGZvciBkaXZpc29ycyBvZiBgbmAuDQogICAgICAgIC50YWtlX3doaWxlKHxkfCBkKmQgPD0gbikgLy8gVGFrZSBlbGVtZW50cyBsZXNzIHRoYW4gb3IgZXF1YWwgdG8gcm9vdCBvZiBgbmAuDQogICAgICAgIC5maWx0ZXIofGR8IG4lZCA9PSAwKSAvLyBUYWtlIG9ubHkgZGl2aXNvcnMuDQogICAgICAgIC5tYXAofGR8IGlmIGQqZCA9PSBuIHsgMSB9IGVsc2UgeyAyIH0pIC8vIENvbnZlcnQgYSBkaXZpc29yIGludG8gdGhlIHNpemUgb2YgdGhlIGdyb3VwIGl0IGJlbG9uZ3MgdG8uDQogICAgICAgIC5mb2xkKDAsIEFkZDo6YWRkKSAvLyBTdW0gdXAgdGhlIGl0ZXJhdG9yLiBgZm9sZGAgZm9sZHMgYW4gaXRlcmF0b3IgYnkgYW4gb3BlcmF0b3IgKHR3by12YXJpYWJsZSBmdW5jdGlvbikuIA0KICAgICAgICAvLyBgQWRkOjphZGQgOiBJNjQgLT4gSTY0IC0%2BIEk2NGAgYWRkcyB0d28gaW50ZWdlcnMuDQopOw0KDQovLyBJbmZpbml0ZSBpdGVyYXRvciBvZiBwb3NpdGl2ZSByYXRpb25hbCBudW1iZXJzIGxlc3MgdGhhbiAxLg0KcmF0aW9uYWxzIDogSXRlcmF0b3IgKEk2NCwgSTY0KTsgLy8gUGFpciBvZiBudW1lcmF0b3IgYW5kIGRlbm9taW5hdG9yLg0KcmF0aW9uYWxzID0gKA0KICAgIEl0ZXJhdG9yOjpjb3VudF91cCgxKSAvLyBJdGVyYXRvciBvZiBkZW5vbWluYXRvcnMNCiAgICAgICAgLm1hcCh8ZHwgKA0KICAgICAgICAgICAgSXRlcmF0b3I6OnJhbmdlKDEsIGQpIC8vIEl0ZXJhdG9yIG9mIG51bWVyYXRvcnMNCiAgICAgICAgICAgICAgICAuZmlsdGVyKHxufCBnY2QobiwgZCkgPT0gMSkgLy8gRmlsdGVyIG91dCBudW1lcmF0b3JzIHdoaWNoIGhhcyBjb21tb24gZmFjdG9yIHdpdGggdGhlIGRlbm9taW5hdG9yLg0KICAgICAgICAgICAgICAgIC5tYXAofG58IChuLCBkKSkgLy8gTWFrZSBwYWlyIG9mIG51bWVyYXRvciBhbmQgZGVub21pbmF0b3IuDQogICAgICAgICkpDQogICAgICAgIC5mbGF0dGVuIC8vIGBmbGF0dGVuIDogSXRlcmF0b3IgKEl0ZXJhdG9yIGEpIC0%2BIEl0ZXJhdG9yIGFgDQopOw0KDQpzdHJpbmdpZnlfcmF0aW9uYWwgOiAoSTY0LCBJNjQpIC0%2BIFN0cmluZzsNCnN0cmluZ2lmeV9yYXRpb25hbCA9IHwobiwgZCl8IG4udG9fc3RyaW5nICsgIi8iICsgZC50b19zdHJpbmc7DQoNCm1haW4gOiBJTyAoKTsNCm1haW4gPSAoDQogICAgZXZhbCAqKHByaW50bG4gJCAiTnVtYmVyIG9mIGRpdmlzb3JzIG9mIDEwMCBpcyAiICsgY291bnRfZGl2cygxMDApLnRvX3N0cmluZyArICIuIik7DQogICAgZXZhbCAqKHByaW50bG4gJCAiRmlyc3QgMTAwIHJhdGlvbmFscyA6ICIgKyByYXRpb25hbHMudGFrZSgxMDApLm1hcChzdHJpbmdpZnlfcmF0aW9uYWwpLmpvaW4oIiwgIikpOw0KICAgIHB1cmUoKQ0KKTsNCg0K)
- For more, see [examples](./examples/).

## Install (macOS / WSL)

- Install [Rust](https://www.rust-lang.org/tools/install).
- Install llvm12.0.1. It is recommended to use [llvmemv](https://crates.io/crates/llvmenv).
    - In macOS, llvmenv installs llvm to "~/Library/Application Support/llvmenv/12.0.1", but llvm-sys currently doesn't understand path with a whitespace correctly, so you need to copy/move "12.0.1" directory to another path.
- Set LLVM_SYS_120_PREFIX variable to the directory to which llvm installed.
- `git clone https://github.com/tttmmmyyyy/fixlang.git && cd fixlang`.
- `cargo install --locked --path .`. Then the compiler command `fix` will be installed to `~/.cargo/bin`.

## Usage

- You can run the source file (with extension ".fix") by `fix run -f {source-files}`.
- If you want to build executable binary, run `fix build -f {source-files}.`.
- For more details, see `fix help`, `fix build --help` or `fix run --help`.
- For debugging, see [this section in Document.md](/Document.md#how-to-debug-fix-program).
- We provide syntax highlight plugin for VSCode. See [this repo](https://github.com/tttmmmyyyy/fixlang_syntaxhighlight).

## Document

* [Document](/Document.md)
* [Built-in Libraries](/BuiltinLibraries.md)
* 紹介（日本語）：[HaskellとRustを足して2で割ったような関数型言語Fixを作っている話](https://qiita.com/tttmmmyyyy/items/ddb1c44efd81e3fc2370)

## Discord

https://discord.gg/ad4GakEA7R
