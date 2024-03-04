## rust-regexps: Rust regular expressions

A Rust implementation of regular expressions (regULAR expRESSIONs)

##### What is Rust ?
- Rust is a computer programming language, in other words it is a way to instruct a computer to perform a task

##### What are regular expressions ?
- Regular expressions is specialized query language which can retrieve text based on some given criteria

For instance, let say I want a list of all timezones names (like UTC+2) in a text file. For this task the regular expression "UTC+[0-9]{1,2}" will do the job, but what does it mean?

It means I want to retrieve any text within my file than contains this character sequence "UTC+" followed by exactly one or two digits

To make things more concrete assume you have a file containing this text:

Cairo has timezone UTC+2

Then applying the above regular expression "UTC+[0-9]{1,2}" will match text
"UTC+2"

Regular expressions are case-sensitive, regular expression "A" will match an uppercase A but it will not match a lowercase A

But wait, that is not the end. Here are some other stuff regular expressions can do:

- Match all telephone numbers in a contacts file.
- Match all dates of the form DD-MM-YYYY (D:Day, M:Month, Y:Year)
- Match all words in a dictionary which start (ex) and end with (e)
- Match all lines containing a given word
- Match all lines longer than a given length
- Match all Arabic words in a file
- Match all English words in a file written in Russian
- Match all blank lines in file
- Match all lines starting with a given phrase

So I hope you get the idea:

- Regular expressions are this special query language you can use to instruct the computer to retrieve some text based on some criteria you want

##### How can I use regular expressions ?

- You can download some computer programs which can "understand" regular expressions, such programs are (GNU grep) and (ripgrep)

##### What is rust-regexps ?
- So, rust-regexps (Rust regular expressions) is an implementation of regular expressions in the Rust programming language. That's, it is a set of instruction to make your computer "understand" regular expressions and perform the text query task

##### Note that regular expressions in and of themselves carry no meaning and they actually do nothing the same way arithmetic expressions do not perform calculation. In other words, regular expressions "express" the computation rather than "performing" the computaiton

##### We can define a regular expression to be a general pattern which describes a set of strings all sharing a certain structure (like containing at least 3 characters)

What is a **string** ? It is a (possibly empty) sequence of characters

Think of it a like string of threads. So a word is string, this whole file is a string and also what is between two consecutive characters is also a string, namely the (empty string) because it has no characters within itself

Usually I will refer to regular expression as "pattern" or just "expression"

Now that you have got to know what regular expression are, it's time to know how this implementation (usually refered to as "computer code" or just "code") works:

#### Two steps:
- Scanning (Tokenizing = producing tokens)
- Parsing (Examining grammartical structure of the expression, if any)

##### Step 1: Scanning
Using the pattern we first introduced in the beginning "UTC+[0-9]{1,2}"

Scanning does a very simple task, it takes your pattern ("regular" expression) and spits tokens which are looser notion of (word). To have less confusion while reading don't cling too much to the ordinary meaning of *word*, rather think of it as an **atomic unit** something that can not be split

Remember, regular expressions are (very specialized) language. You can't speak a language without *words* and the ***tokens*** coming from **Step 1** are exaclty those *words* of regular expressions

That's it, in **Step 1** your computer creates (and stores) a list of the *words* you used in your "regular expression sentence" if to say


##### Step 2: Parsing
Parsing is a fancy word meaning "determining the grammatical structure of a regular expression". Yes, regular expressions have grammar and hence not every sequence of characters (string) is a *valid* regular expression

Why we need a grammar for something that's not even a *natural language* like Engligh?

Without grammar you yourself would've not been able to reach this part of this file because the whole file is just a sequence of letters grouped together in paragraphs with each word standing on its own making no relation to any other word

###### This is exactly how your learn a new language, first you're taught words (scanning) and then you're taught how group those words (*tokens*) together to make meaningful sentences (parsing)

###### No cleverness, we tell the computer to do the same every time it's given a regular expression. In fact this is how computers "understand" what we humans refer to as "code"

##### *For futher information on this subject, you can read:*
- ***Compilers: Principles, Techniques, and Tools*** also known as ***The Dragon Book*** by:
    - Alfred Aho
    - Ravi Sethi
    - Jeffrey Ullman
    - Monica S. Lam
- [***Crafting Interpreters***](https://craftinginterpreters.com)
- [***Writing An Interpreter in Go***](https://interpreterbook.com)
- and its sequel [***Writing a Compiler in Go***](https://compilerbook.com)

*Last three are more (practice oriented)*
