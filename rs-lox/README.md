# Crafting Thoughts & TILs
Following [Crafting Interpreters](https://craftinginterpreters.com/) in Rust. This is nice, I have to rethink how I implement everything, since original is in C and Rust provide much more convenience on one hand and challenges with the safety constraints. This is great both for understanding Rust and how to craft interpreters.

At first I aim to just make the thing run to spec. Later on will try to optimize some parts of it.
## Aug 15, 2022
* how should I split the interpreter to crates? Best would be crates that provide services to others. So the `lang` crate is great since it doesn’t need anything and provides tokens and scanner to everyone else. but everyone else need the lang crate. So I guess that I need to avoid circular dependencies, that will be the sign that I did something wrong. compiler crate will have the parser and compiler in there. 
* I skip blank spaces when tokenizing, this makes my error citation code fail. Need to keep track of blanks spotted so far in the scanner.
## Aug 14, 2022
* RC pointers are perfect for string interning. But I kinda want to have more features in the VM before I go and implement optimizations. To see how big of an improvement I can really get there.
*  `cargo t -- —no capture` to dump prints and dbg! During lib testing
* Debugging annoying bug, it seems that print statement doesn’t put global variable on the stack. 
	* I didn’t handle the NIL OpCode and posed instead of peeked for global variable setting 🙃 🤡 
		* That’s why my pop operation was failing…
* Going to try and play with cargo workspaces. Want to separate my code into three crates, runtime (vm, opcode, 
## Aug 13, 2022
* Using `&str` as key for my global var store.
	* I store identifiers as heap strings, inside OpCode chunks. To prevent string copies, I hash the string slice (which is a pointer with length) of each identifier. 
	* I didn’t want to mess with string interning, but looks like I don’t have a choice. For now I gonna do the string cloning but to make previous point work I need interning.
	* I’m in a cloning hell right now
* 
## Aug 12, 2022
* **Representing Heap Values**
	* In C you have to allocate memory for strings on the heap but in Rust, String does that for me and gets me back a pointer, length and capacity. 
	* I simply use rust’s heap instead of redoing the heap allocation myself. I might pay with perf on this. But it’s much simpler and leaves room for future optimizations