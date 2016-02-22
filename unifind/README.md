#A simple library and command line utility to search for unicode characters.

Usage:
```
$ unifind <term1> <term2> ...
```

#Where does the data come from?
The sub-crate unifind-generator can be used to generate src/generated.rs
That crates generates a trie for fast lookup of the names in nUnicodeData.txt (copyrighted by unicode consortium).

This file is published at http://www.unicode.org/Public/UCD/latest/ucd/UnicodeData.txt
