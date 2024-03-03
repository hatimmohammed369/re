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
