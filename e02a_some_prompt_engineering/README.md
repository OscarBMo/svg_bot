# First iteration of prompt engineering

On this experiment, we try to improve the generator by using a more specific prompt.

We also remove the prompt from the code and put it into its own .md file, which is a nicer way to handle it. Then we use `include_str!` to read the prompt from the file and use it in the code together with some string replacement to fill in the variables.