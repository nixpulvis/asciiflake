# asciiflake

Draw random snowflakes in the terminal.

```sh
cargo install --git https://github.com/nixpulvis/asciiflake
```

```
$ asciiflake --size 4 --id 1 --count 3 --verbose
    \       /
     \     /
      \ * /
     *-* *-*
------*-*-*------
     *-* *-*
      / * \
     /     \
    /       \

id 1

    \       /
     \ / \ /
    --*   *--
   \   \ /   /
----*---*---*----
   /   / \   \
    --*   *--
     / \ / \
    /       \

id 2

    \       /
     \ / \ /
    --* * *--
   \ *-* *-* /
----*-*-*-*-*----
   / *-* *-* \
    --* * *--
     / \ / \
    /       \

id 3
```

## rustflake

`rustflake` lays a Rust program out along the branches of a snowflake. The
result is still the same program: tokens are only moved around, leftover room
goes to extra spaces between tokens or `/***/` comments, ordinary comments are
dropped, and doc comments become `#[doc]` attributes. A token longer than the
stroke is wide spills past its edge. The flake is only as big as the program
needs, down to a smallest size set by `--thickness`, so a short program on
thick strokes is mostly comments.

```
$ rustflake --size 2 --id 1 --thickness 2 hello.rs
         fn /***/ main(
       /***********/){
     /************/for i
   /*******/in /********/1
    /**************/..=3{
  /**************/println!(
/***************************/
  /***********************/
    "{} snowflake{}",i,if
   /********/i /******/==1
     /*************/{""}
       /********/else{
      "s"}) /***/ /***/
        ;     }     }
```

`metaquineflake` prints `quineflake`, a quine drawn this way. The quine is a
cargo script (which needs nightly) that depends on this repository's
`fmt-n-quine` branch, so it builds itself anywhere. Running it prints its own
source laid out on the size 5 flake with the next ID, which is also the program
that prints the one after that. After all 36 IDs it's back where it started.

```sh
cargo run -q --bin metaquineflake > quineflake.rs
chmod +x quineflake.rs
./quineflake.rs > next.rs
```

This script runs one step at a time and prints the result. The first run makes
`target/quineflake.rs` from the metaquine, and each run after that replaces it
with the next one:

```sh
scripts/quineflake.sh
```

## License

MIT
