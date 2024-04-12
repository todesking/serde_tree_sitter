# Serde deserializer for Tree-sitter

Deserializer for `tree_sitter::Node`.
You can map tree-sitter's parse result to Rust struct/enum with `#[derive(serde::Deserialize)]`.

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

serde_tree_sitter::from_tree::<Expr>(tree, src, true)
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
serde_tree_sitter::from_tree::<Vec<u32>>(tree, src, true)
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
