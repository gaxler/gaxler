# Crafting TILs
Following [Crafting Interpreters](https://craftinginterpreters.com/) in Rust. This is nice, I have to rethink how I implement everything, since original is in C and Rust provide much more convenience on one hand and challenges with the safety constraints. This is great both for understanding Rust and how to craft interpreters.

At first I aim to just make the thing run to spec. Later on will try to optimize some parts of it.

## Aug 13, 2022

* **Using `&str` as key for my global var store.**
	* I store identifiers as heap strings, inside OpCode chunks. To prevent string copies, I hash the string slice (which is a pointer with length) of each identifier.