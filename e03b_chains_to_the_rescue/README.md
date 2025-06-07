# Some more prompt engineering

## Idea

On the first part of e03, we could already notice some major issues with some of the prompts:

- The design prompt has a tendency to produce text for the titles, and this is not something that we want unless we explicitly ask for it.
- The svg code review prompt was not explicit enough and it is producing something that I didn't want: just a review itself. What I wanted was the original code with the results of the review applied to it. This highlights the importance of being crystal clear about what we want to achieve with the prompt.

Let's see to what extent we can improve upon the current situation just by iterating on the prompts.

On the other hand, we can also observe that one common problem that we had before (the backticks ```svg``` delimiting the code) is not present anymore. This might be thanks to the decoupling of the design and the generation steps, which allow us to have more clear instructions on what we want to achieve.
