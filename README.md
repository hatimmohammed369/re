## rust-regexps: Rust regular expressions

A Rust implementation of regular expressions (**reg**ular **exp**ression**s**, hence the name **regexps**).

##### What is Rust ?
- **Rust** is a computer programming language, in other words it is a way to instruct a computer to perform a task.

##### What are regular expressions ?
- **Regular expressions** is a specialized query programming language which can instruct the computer to retrieve text based on some given criteria.

For instance, let's say I want a list of all timezones names (like **UTC+2**) in a text file. For this task the regular expression `UTC[+][0-9]{1,2}` will do the job, but what does it mean?

It instructs the computer to retrieve each text within my file than contains this character sequence *UTC+* followed by exactly one or two digits.

To make things more concrete assume you have a file containing this text:

*Cairo has timezone UTC+2*

Then applying the above regular expression `UTC[+][0-9]{1,2}` will match text
*UTC+2*.

But wait, that is not the end. Here are some other stuff regular expressions can do:

- Match all telephone numbers in a contacts file.
- Match all dates in a particular format you want.
- Match all words in a dictionary starting with (ex) and ending with (e).
- Match all lines containing a given word.
- Match all lines longer than a given length.
- Match all Arabic words in an English document.
- Match all blank lines in a file.
- Match all lines starting with a given phrase.
- Describe **lexemes** of a computer language.

So I hope you get the idea:

- Regular expressions are this special query language you can use to instruct the computer to retrieve some text based on some criteria you want.

##### How can I use regular expressions ?

- You can install a regular expressions search tool, some are:
    - [GNU grep](https://www.gnu.org/software/grep/)
    - [ripgrep](https://github.com/burntsushi/ripgrep)
    - [ack](https://beyondgrep.com)

just to name a few. Google `grep-like tools` for more

##### What is rust-regexps ?
- So, rust-regexps (Rust regular expressions) is an implementation of regular expressions in the Rust programming language. That's, it is a set of instruction to make your computer *understand* regular expressions and perform the text query task.

##### Note that regular expressions in and of themselves carry no meaning and they actually do nothing the same way arithmetic expressions do not perform calculation. In other words, regular expressions "express" the computation rather than "perform" it.

##### We can define a regular expression to be a general pattern which describes a set of strings all sharing a certain structure (like containing at least 3 characters).

What is a **string** ? It is a (possibly empty) sequence of characters.

Think of it a like string of threads. So a word is string, this whole file is a string and also what is between any two consecutive characters is also a string, namely the (empty string) because it has no characters within itself.

Usually I will refer to regular expression as *pattern* or just *expression*.

Now that you have got to know what regular expression are, it's time to know how this implementation (usually refered to as "computer code" or just "code") works:

#### *An Introduction to Programming Languages and Compilers*

You can define a **programming langauge** as method of instructing the computer to perform a task. Well, no computer does actually *understand* a programming language, but how come computers do all these different things?

This is where **compilers** come into play, they play the role of an *interpreter* between two groups of people who speaks different langauges.

You can regard the first group as *the programmers*, those people who make computers do all these different sorts of things. The second group is computers themselves.

You can view **regular expressions** as a very specialized *programming language* we can use to retrieve text based on some criteria we want.

#### Compilers, in general, work in three major steps:

- Scanning
    - Transform their input into **tokens**.
- Parsing
    - Examine the **grammatical structure** of tokens from previous step.
- Target generation
    - Generating an *equivalent*, if to say, representation of the input in some other form or just optimize representation of the input itself.

##### Performing these three steps in the context of regular expressions:
##### Step 1: Scanning
Using the pattern we first introduced in the beginning `UTC+[0-9]{1,2}`

Scanning does a very simple task, it takes your pattern (*regular* expression) and spits **tokens**.
 - A **Token** is a **lexeme** along with some data.
 - A **lexeme** is a looser notion of (word), for instance `2000` and `x` are valid lexemes in most programming languages even though you would not regard `2000` or `x` as *valid words*.

To have less confusion while reading don't cling too much to the ordinary meaning of *word*, rather think of it as an **atomic unit** something that can not be split.

A little example here might help, `x` can be regarded as a **lexeme** but when add to its descriptions the notion of being a *variable name* it becomes a **Token**.

Remember, regular expressions are (very specialized) language. You can't speak a language without *words* and the ***tokens*** coming from **Step 1** are exaclty those *words* of regular expressions.

That's it, in **Step 1** your computer creates (and stores) a list of the *words* *(actually Tokens)* you used in your "regular expression sentence" if to say.

##### Step 2: Parsing
Parsing is a fancy word meaning **determining the grammatical structure of tokens**. Yes, computer langauges have grammar and hence not every sequence of characters (string) is a *valid sentence*, in our case a valid regular expression.

Why we need a grammar for something that's not even a *natural language* like Engligh?

Without grammar you yourself would've not been able to reach this part of this file because the whole file is just a sequence of letters grouped together in paragraphs with each word standing on its own making no relation to any other word.

###### This is exactly how your learn a new language, first you're taught words (scanning) and then you're taught how group those words (*tokens*) together to make meaningful sentences (parsing).

###### No cleverness, we tell the computer to do the same every time it's given a regular expression. In fact this is how computers "understand" what we humans refer to as "code".

##### Step 3: Target Generation
Computer programs **MUST** have a precise fixed, if to say, interpreteration.

A program which behaves differently ***without computing what it was made for*** each time it's envoked is very unlikely to be useful.

That is, what we want is the same outcome for the same input each time the computer carries out **Step 1** and **Step 2**. In other words **Step 1** and **Step 2** **MUST ALWAYS** map the same input to the same output.

This step the compiler generates that *other representation* of the input because that *other representation* is unique to each input.

No need to redo **Step 1** and **Step 2** each time for the same input, instead the compiler generates and stores the *other representation* of the input.

##### *For further information on this subject:*
##### More theory oriented:
- ***Introduction to the Theory of Computation***, an excellent book
    - Michael Sipser
- ***Introduction to Automata Theory, Languages and Computation***
    - John E. Hopcroft
    - Rajeev Motwani
    - Jeffrey D. Ullman
- ***Mastering Regular Expressions***:
    - Jeffrey E. F. Friedl
- ***Compilers: Principles, Techniques, and Tools*** also known as ***The Dragon Book***:
    - Alfred Aho
    - Ravi Sethi
    - Jeffrey Ullman
    - Monica S. Lam
##### More practice oriented:
- [***Crafting Interpreters***](https://craftinginterpreters.com)
    - Robert Nystorm
- [***Writing An Interpreter in Go***](https://interpreterbook.com)
- and its sequel [***Writing a Compiler in Go***](https://compilerbook.com)
    - by Thorsten Ball
