# Serde deserializer for Tree-sitter

## Example

### Map node to enum
```javascript
// grammar.js
rules: {
  expr: $ => choice($.expr_int, $.expr_bool)
}
```

```rust
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
enum Expr {
    ExprInt(i64),
    ExprBool(bool),
}

let tree: tree_sitter::Tree = parse("...")?;

serde_tree_sitter::from_tree::<Expr>(tree, src)
```

### Map named children to tuple struct

```javascript
// grammar.js
rules: {
  let: $ => seq('let', $.ident, '=', $.expr),
  // ...
}
```

```rust
#[derive(serde::Deserialize, Debug)]
#[serde(rename="let")]
struct Let(Ident, Expr);
```

### Map named children to vec

```javascript
rules: {
  nums: $ => repeat(seq($.int, ',')),
  // ...
}
```

```rust
serde_tree_sitter::from_tree::<Vec<u32>>(tree)
```

## Map field to struct
```javascript
rules: {
  object: $ => seq('{', repeat(seq($.pair, ',')), '}'),
  pair: $ => seq(field('key', $.ident), ':', field('value', $.expr),
  // ...
}
```

```rust
#[derive(serde::Deserialize, Debug)]
#[serde(rename="object")]
struct Object(Vec<Pair>);

#[derive(serde::Deserialize, Debug)]
#[serde(rename="pair")]
#[serde(rename_all="snake_case")]
struct Pair {
    key: String,
    value: Expr,
}
```

## Supported types

### Root types

* `()`
* `String`
* `&str`
* Number types: `(u|i)(8|16|32|64)` and `f(32|64)`
* `bool`
* Unit struct: `struct Foo;`
* Newtype struct: `struct Foo(T);` where `T` is tuple member type.
* Tuple struct: `struct Foo(R1, R2, ..., RN)`
    * If you want to match exact one child node, use newtype struct with one-ary tuple `struct Foo((R,))`
* Struct: `struct Foo { f1: F1, f2: F2, ..., fn: FN }` where `F` is field member type.
* Enum: `enum Foo { ... }`:
  * `UnitVariant`
  * `TupleVariant(C1, ..., CN)`
  * `StructVariant{f1: C1, ..., fn: CN}`
* Tuple: `(T1, T2, ..., TN)`
* `Vec<R>`
* `Box` of any root type

### Tuple member types

* Any root type `R`: Matches against parent node
* `(R1, R2, ..., RN)`: Matches named children. Required exact N children.
* `Vec<R>`: Matches named children.
* `Option<R>` Matches 0 or 1 named child.

### Field member types

* Any root type `R`: If there is exact one node in the field, matches against it.
* `(R1, R2, ..., RN)`: Matches named children in the field. Required exact N children.
* `Vec<R>`: Matches named children in the field.
* `Option<R>` Matches 0 or 1 named child in the field.
