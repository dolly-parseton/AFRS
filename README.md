# AFRS

Another f****** rule syntax, simple quick pattern matching on JSON objects (more data types to come).

## Rule Synax

|Field|Field Name|Description|
|---|---|---|
|Name|`name`|The name of the rule.|
|Variables|`variables`|One or more variables that matches data in a field. Each variable object needs to have a `name` field and a `field` field, the `name` field needs to match a variable name in the conditional string. The `field` field has to match a fieldname in the JSON object, follows the [`gjson`](https://github.com/tidwall/gjson) sytnax. Lastly the `type` field has to match one of the variable kinds in the table below.|
|Conditional|`conditional`|A string comprised of the variable names (For example `A and B | C`)|

## Variables

|Type|Additional Rule Field(s)|Description|
|---|---|---|
|Contains|`contains`|Variable type looks to see if the value at the location specified by `field` contains the value provided in the `contains` field.|
|Exact|`exact`|Variable type looks to see if the value at the location specified by `field` exactly matches the value provided in the `exact` field.|
|Regex|`regex`|The `regex` field is read in and deserialized as a [`Regex`](https://docs.rs/regex/1.5.4/regex/struct.Regex.html) pattern, this is then matched against the value at the location specified by `field`.|
|Compare|`ordering`,`value`|The `ordering` field is read in and deserialized as a [`Ordering`](https://doc.rust-lang.org/std/cmp/enum.Ordering.html), `value` is read as a double. `value` is compared to the value pulled at `field`.|

<!-- 
## `afrs-cli`

Install using `cargo install afrs` 


-->