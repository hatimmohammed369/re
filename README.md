A regular expressions library written in rust

Regular expressions objects are constructed in a three-stage process:
Scanning
Parsing
Compiling

Scanning: Turning the source pattern string into a stream of tokens
Parsing: Turning the stream of tokens into a structured syntax tree
Compiling: Turning the syntax tree into executable instructions performing the actual matching

Scanning:
The scanner reads source string one character a time generating a Token object (actually Option<Token>)
when it reaches end of input it returns None

Parsing:
The parser requests one token at a time and uses recursive decent
