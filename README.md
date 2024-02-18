# RUST-GREP
Check if input matches a regex pattern.

## Installation
Install rust in your machine following the [documentation](https://www.rust-lang.org/tools/install).

## Usage
Create an executable using
```
cargo build
```

Execute the program
```
./target/debug/rust-grep regex-expression
```

### Examples
```
cat src/main.rs | ./target/debug/rust-grep "fn.*->.*Vec<u8>" # Functions returning a Vec<u8> 
```

```
echo "Hello world" | ./target/debug/rust-grep "Hello"
```
