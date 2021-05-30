# AFRS

Another fucking rule syntax, simple quick pattern matching on JSON objects (more data types to come).

## Rule Synax

* Name, the name of the rule.
* Variables, one or more variables that matches data in a field. Each variable object needs to have a `name` field and a `field` field, the `name` field needs to match a variable name in the conditional string. The `field` field has to match a fieldname in the JSON object, follows the [`gjson`](https://github.com/tidwall/gjson) sytnax.
    * Regex: the `regex` field can contain valid regex string.
    * Exact: the `data` field needs to match exactly the data in the JSON object.
* Conditional, a string comprised of the variable names. For example `A and B | C`
