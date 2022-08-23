# Crafting Thoughts & TILs
Following [Crafting Interpreters](https://craftinginterpreters.com/) in Rust. This is nice, I have to rethink how I implement everything, since original is in C and Rust provide much more convenience on one hand and challenges with the safety constraints. This is great both for understanding Rust and how to craft interpreters.

At first I aim to make the thing run to spec. Later on will try to optimize some parts of it.
## Aug 23, 2022
* my problem with the local while implementation was that I pop on set_local. This breaks my assumption that the cleanup is done in statement end. So need to keep this invariant. No popping from the stack unless its through a POP opcode
* `std::mem::discriminant` for enum variant comparison
* split parser implementation to a large module, the `parser.rs` file was getting to big to handle.
## Aug 18, 2022
* I realized that the Parser is really cumbersome, it didn’t translate well to Rust. Maybe because the original was intended for a book where each line of code is going to be explained. Anyway, I think I can make it clearer by redoing it as a state machine.
## Aug 17, 2022
* maybe there is a way to define stack consistency in Rust type system?
* How if statements should look like?
	* there is the if keyword, followed by a boolean result expression, this thing goes on top of the stack. Next I read the stack and move my instruction pointer to the if true block or else block
	* if true block needs to have a jump to skip the else block
	* true block address is the length of chunk at the moment we ended reading the expression.
	* else block address is the chunk length at the end of writing the else block
	* one we know it, we need to go back and define the jump address to skip the else block
* I cause stack underflows with my AND OpCode optimization, need to rethink the process of how it works and build some debug snippets to test changes.
## Aug 16, 2022
* There is no reason to try and mix &str and String when I plan eventually to refactor the way I handle identifiers. Switched everything to string cloning and now local vars work.
## Aug 15, 2022
* got stuck on fixing the identifier global local confusion. Its too late anyway
* placed value stores (Stack, VarStore, Chunk) in the values crate. This crate provides services to the runtime and compile time. 
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
## Aug 12, 2022
* **Representing Heap Values**
	* In C you have to allocate memory for strings on the heap but in Rust, String does that for me and gets me back a pointer, length and capacity. 
	* I simply use rust’s heap instead of redoing the heap allocation myself. I might pay with perf on this. But it’s much simpler and leaves room for future optimizations
	* Some [experiments](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=d4371f3475b6a005a3392ef8687f52e6) on how to keep string as a raw pointer