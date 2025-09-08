# Expression Language

Stylus uses a custom expression language for evaluating conditions in monitor configurations. This language supports arithmetic operations, logical operations, string manipulation, and comparisons.

## Overview

The expression language is currently only used in the SNMP and ping monitors.

## Data Types

The expression language supports two main data types:

- Integers: Whole numbers (e.g., `42`, `-17`, `0`)
- Strings: Text values enclosed in quotes (e.g., `"hello"`, `'world'`)

## Literals

### Numbers
```javascript
42        // Positive integer
-17       // Negative integer
0         // Zero
```

### Strings
Strings can be enclosed in single or double quotes:
```javascript
"hello world"    // Double quotes
'hello world'    // Single quotes
```

### Escape Sequences
Strings support escape sequences:
```javascript
"hello \"world\""    // Escaped double quote
'hello \'world\''    // Escaped single quote
"hello \\world\\"    // Escaped backslash
```

### Boolean Values
```javascript
true      // Boolean true (evaluates to 1)
false     // Boolean false (evaluates to 0)
```

## Arithmetic Operations

### Basic Arithmetic
```javascript
a + b     // Addition (numbers) or concatenation (strings)
a - b     // Subtraction
a * b     // Multiplication
a / b     // Division
a ^ b     // Exponentiation (a raised to power b)
-a        // Negation
```

### Examples
```javascript
2 + 3           // 5
"hello" + " " + "world"    // "hello world"
10 - 3          // 7
4 * 5           // 20
15 / 3          // 5
2 ^ 3           // 8
-5              // -5
```

## Comparison Operations

### Comparison Operators
```javascript
a == b    // Equal to
a != b    // Not equal to
a > b     // Greater than
a < b     // Less than
a >= b    // Greater than or equal to
a <= b    // Less than or equal to
```

### Examples
```javascript
5 == 5           // true
"up" == "down"   // false
10 > 5           // true
"abc" < "def"    // true
7 >= 7           // true
3 <= 10          // true
```

## Logical Operations

### Logical Operators
```javascript
a and b   // Logical AND
a or b    // Logical OR
not a     // Logical NOT
```

### Examples
```javascript
true and true    // true
true and false   // false
true or false    // true
false or false   // false
not true         // false
not false        // true
```

## String Functions

### String Manipulation
```javascript
startswith(str, prefix)    // Check if string starts with prefix
endswith(str, suffix)      // Check if string ends with suffix
contains(str, substr)      // Check if string contains substring
length(str)                // Get string length
```

### Examples
```javascript
startswith("hello world", "hello")    // true
endswith("hello world", "world")      // true
contains("hello world", "lo wo")      // true
length("hello")                       // 5
```

## Type Conversion Functions

### Type Conversion
```javascript
str(value)    // Convert value to string
int(value)    // Convert value to integer
```

### Examples
```javascript
str(42)       // "42"
int("123")    // 123
str(true)     // "1"
int("abc")    // 0 (default for failed conversion)
```

## Precedence and Associativity

The expression language follows Python-like precedence rules (from lowest to highest):

1. `or`
2. `and`
3. `not`
4. Comparisons (`==`, `!=`, `>`, `<`, `>=`, `<=`)
5. Addition/Subtraction (`+`, `-`)
6. Multiplication/Division (`*`, `/`)
7. Exponentiation (`^`)
8. Functions, parentheses, literals, variables

### Examples
```javascript
a == 1 and b == 2 or c == 0    // ((a == 1) and (b == 2)) or (c == 0)
not a == 1                      // not (a == 1)
2 + 3 * 4                       // 2 + (3 * 4) = 14
2 ^ 3 + 1                       // (2 ^ 3) + 1 = 9
```

## Truthiness

Values are considered "truthy" or "falsy" in logical operations:

- Falsy values: `0`, `""` (empty string), `false`
- Truthy values: Any non-zero number, any non-empty string, `true`

### Examples

```javascript
0 and "hello"      // false (0 is falsy)
1 and "hello"      // true (both are truthy)
"" or 42           // true (42 is truthy)
not 0              // true
not "hello"        // false
```

## Context Variables

Depending on the context, the expression language has access to different
local variables.

### SNMP Examples

The [SNMP monitor](monitor/snmp.md) makes the OIDs for each interface available
as context variables.

```javascript
// Check if interface is up and admin enabled
ifOperStatus == "up" and ifAdminStatus == "up"

// Check if interface is Ethernet and not loopback
ifType == "ethernetCsmacd" and not contains(ifDescr, "Loopback")

// Check if interface speed is less than 1Gbps
ifSpeed < 1000000000

// Check if interface description contains specific text
contains(ifDescr, "10G Ethernet Adapter")
```
