# Chapter 1

## Writing and Running a Fluid Program
Lets create a source file and call it *main.fluid*. Now open the *main.fluid* file you just created and enter the following:

<span class="filename">Filename: main.fluid</span>

```
function main(argc: number, argv: string[]) -> number {
    print("Hello, World!");
    
    return 0;
}
```

<span class="caption">A program that prints `Hello, world!`</span>

Save the file and then open a terminal window. Enter the following command to run the file:

```bash
$ fluid run main.fluid
Hello, World!
```
