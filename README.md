A regular expressions library written in rust

Regular expressions objects are constructed in a three-stage process:
Scanning
Parsing
Compiling

Scanning: Turning the source pattern string into a stream of tokens
Parsing: Turning the stream of tokens into a structured syntax tree
Compiling: Turning the syntax tree into executable instructions performing the actual matching
