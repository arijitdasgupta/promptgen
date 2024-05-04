# promptgen

A low dependency parser for a simple promptgen DSL that allows questions answer style interactions. Useful for interactive birthday cards and such.

## promtgen lang
```
> (NO) "Are you a human?"
< (YES) "Yes, I am"
< (ANS_NO) "No"
> (ANS_NO) "That's very weird! Care to try again?"
< (NO) "Please!"
> (YES) "Nice! Glad to meet you human!"
```

### Grammer
 - The starting angular bracket is decides whether it's a question or answer. Right angular bracket `>` is question, left angular bracket `<` is answer.
 - Labels are like pointers to specific questions from answers. Looks like `(LABEL_1)`. `()` is also a valid label. These can't contain whitespaces.
 - The answer and question texts are represented with a `"Stuff!"`. 
 - Multiline question or answer text is possible, depending on how the parser is used.

### Rules
 - The prompter system starts with the first question as the starting question.
 - If an answer has a label, answering with that answer will go to the question with the corresponding label.
    - If the label doesn't exist as a question symbol, the system will go to the next available question.
 - If a question doesn't have an answer associated with it, the system won't go any further.

# Components
### Prompter
 - [ ] IMPLEMENT!
### Parser
 - [ ] IMPLEMENT!
### Lexer
#### Tasks
 - [ ] Better reporting on where the error happened by reporting faulty position in the string.
  - Can be done by wrapping convert one error style into another, and having a global error enum to represent global failure indices.