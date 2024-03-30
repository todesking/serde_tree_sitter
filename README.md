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

serde_tree_sitter::from_tree::<Expr>(tree) 
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

### Root type
* `()`
* `String`
* `u8, ..., u64, i8, ..., i64, f32, f64, bool`
* `struct Foo;`
* `struct Foo(C);` where `C` is children type.
* `struct Foo { f1: C1, f2: C2, ..., fn: CN }`
* `enum Foo { ... }`:
  * `UnitVariant`
  * `TupleVariant(C1, ..., CN)`
  * `StructVariant{f1: C1, ..., fn: CN}`
* `Box` of any root type

### Children type
* Any root type `T`
* `(T1, T2, ..., TN)`
* `Vec<T>`
* `Option<T>`
