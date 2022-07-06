# Filling a Niche

> **Note:** I am not currently accepting pull requests.

This is the artifact for my [POPL 2022 Student Research Competition][popl] project,
["Filling a Niche: Using Spare Bits to Optimize Data Representation"][paper].
You can read the 3-page extended abstract [here][paper], and you can watch the
3-minute lightning talk [here][talk].

[popl]: https://popl22.sigplan.org/track/POPL-2022-student-research-competition
[paper]: https://www.noahlev.org/papers/popl22src-filling-a-niche.pdf
[talk]: https://youtu.be/dROaEavjEQw

This artifact implements a type layout engine and interpreter for a simple,
purely-functional language. See [below][usage] for instructions on how to use
the interpreter.

## Building and Running

1. Install Rust.
2. Run `cargo run` in the repository to launch the interactive interpreter.

## Usage

[usage]: #usage

If you enter an expression into the interpreter, it will compile it to LIR (a
low-level IR; like a functional LLVM IR), evaluate it, and print the resulting
*LIR* machine value (*not* the equivalent high-level, abstract value). For example, the following code demonstrates the niched representation of `None` of type `Maybe<Bool>`:

```
> alias MaybeBool = < None of {} | Some of < False of {} | True of {} > > in <None = {}> as MaybeBool
{ tag = 2_u64 }
```

You can find some example expressions in `example.fun`. (Note that multiline input is not currently supported by the interpreter, so you will have to replace newlines within an expression with spaces.)

The interpreter also has several commands that you can use to introspect an
expression or type. For example, `:lyt` will print the computed memory layout of
a type:

```
> :lyt < None of {} | Some of < False of {} | True of {} > >
Variant(Tagged(tag: Niche(path: ({root} as(transparent) Some).{tag}, values: { None => 2 }), variants:
| None => Aggregate {}
| Some => Variant(Tagged(tag: Direct(values: { False => 0, True => 1 }, niches: 3..=18446744073709551615), variants:
| False => Aggregate {}
| True => Aggregate {}
))
))
```

| Command                | Argument   | Description                                                                  |
|------------------------|------------|------------------------------------------------------------------------------|
| `:hir`                 | expression | Print the HIR (high-level IR; desugared syntax)                              |
| `:lir`                 | expression | Compile to LIR and print it (low-level IR; like a functional LLVM IR)        |
| `:lyt`, `:layout`      | type       | Print the type's layout                                                      |
| `:t`, `:hty`, `:hirty` | expression | Print the type of the expression's HIR form                                  |
| `:lty`, `:lirty`       | expression | Compile to LIR and print the LIR type                                        |
| `:size`                | type       | Print the packed size, in bytes, of a type (i.e., the size ignoring padding) |
