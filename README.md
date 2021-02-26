<img src="misc/logo.png" width="150px">

[lines-of-code-badge]: https://tokei.rs/b1/github/fluid-lang/fluid?category=code

[repo]: https://github.com/fluid-lang/fluid

[discord]: https://discord.gg/AAv4gURK2S
[discord-badge]: https://img.shields.io/discord/807452320996261889

[![Join us on Discord][discord-badge]][discord]
[![Lines of Code][lines-of-code-badge]][repo]

# Fluid Programming Language

> ⚠⚠⚠ Fluid is in a very early stage of developement and should not be used for production use. ⚠⚠⚠

## Welcome to Fluid
Fluid is a new fast, open source, statically typed programming language with modern syntax without sacrificing performance, that makes it easy to build simple, reliable, and efficient software.

### What does Fluid code look like?
```
function main(argc: number, argv: string[]) -> number {
    print("Hello, World!");
    
    return 0;
}
```

To learn more about the programming language, visit [fluid-lang.github.io](https://fluid-lang.github.io)

- [Contributing to Fluid](#contributing-to-fluid)
- [Getting Started](#getting-started)

## Getting Started
### Setup
Fluid uses a small python script called fluid.py to simplify the process of building fluid. More information about fluid.py can be found by runnning 
```bash
$ python ./fluid.py --help
```

1. Make sure you have installed the dependencies:
    
    * `rust`
    * `python` 3 or 2.7
    * `git`
    * `llvm-dev` 11.x

2. Clone the [source](https://github.com/fluid-lang/fluid) using git

    ```bash
    $ git clone https://github.com/fluid-lang/fluid
    ```

3. Build and run fluid
    
    ```bash
    $ python ./tools/fluid.py build [release | debug] --run
    ```

## Contributing to Fluid
Contributions are absolutely, positively welcome and encouraged! Contributions come in many forms. You could:

1. Submit a feature request or bug report as an [issue](https://github.com/fluid-lang/fluid/issues).
2. Contribute to the code via [pull requests](https://github.com/fluid-lang/fluid/pulls).

We aim to keep the code quality at the highest level. This means that any
code you contribute must be:

  * **Commented:** Complex and non-obvious functionality must be properly
    commented.
  * **Styled:** Your code's style should match the rust code style.
    style.
  * **Tested:** You must write (and pass) convincing tests for any new
    functionality.
