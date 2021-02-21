use fluid_codegen::{CodeGen, CodeGenType};
use fluid_lexer::Lexer;
use fluid_parser::Parser;

use ansi_term::Colour;
use rustyline::Editor;
use structopt::StructOpt;

use std::{error::Error, fs::File, io::Read, path::Path, process};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const HELP: &str = "At the prompt you can type Fluid Code or type repl commands preceded by a `.`

    .reset => Reset the codegen context.

For more information about fluid commands `fluid --help`";

#[derive(Debug, StructOpt)]
enum Command {
    Run {
        path: String,
    },
    Build {
        path: String,

        #[structopt(long, short)]
        emit_llvm: bool,
    },
}

#[derive(Debug, StructOpt)]
struct CLI {
    #[structopt(subcommand)]
    command: Option<Command>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = CLI::from_args();

    match args.command {
        Some(command) => match command {
            Command::Run { path } => run_file(path)?,
            Command::Build { path, emit_llvm } => build_file(path, emit_llvm)?,
        },
        None => repl()?,
    }

    Ok(())
}

fn run_file(path: String) -> Result<(), Box<dyn Error>> {
    let mut file = File::open(&path)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    let mut codegen = CodeGen::new(&path, CodeGenType::JIT { run_main: true });

    let mut lexer = Lexer::new(contents, path);
    let tokens = match lexer.run() {
        Ok(tokens) => tokens,
        Err(errors) => {
            for err in errors {
                println!("{}", err);
            }

            process::exit(1);
        }
    };

    let parser = Parser::new(tokens);

    codegen.run(parser);
    codegen.free();

    Ok(())
}

fn build_file(path: String, emit_llvm: bool) -> Result<(), Box<dyn Error>> {
    let mut file = File::open(&path)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    let mut lexer = Lexer::new(&contents, &path);
    let tokens = match lexer.run() {
        Ok(tokens) => tokens,
        Err(errors) => {
            for err in errors {
                println!("{}", err);
            }

            process::exit(1);
        }
    };

    let parser = Parser::new(tokens);

    if emit_llvm {
        let mut codegen = CodeGen::new(&path, CodeGenType::JIT { run_main: false });

        codegen.run(parser);
        codegen.emit_llvm(&path);
        codegen.free();
    } else {
        let mut codegen = CodeGen::new(&path, CodeGenType::JIT { run_main: false });
        let path = Path::new(&path);

        codegen.run(parser);

        if let Some(parent) = path.parent() {
            let file_name = path.file_name().unwrap().to_string_lossy().replace(".fluid", ".obj");

            let out = parent.join(file_name);
            codegen.emit_object(&out);
        } else {
            let file_name = path.file_name().unwrap().to_string_lossy().replace(".fluid", ".obj");

            let out = Path::new(&file_name);
            codegen.emit_object(&out);
        }

        codegen.free();
    }

    Ok(())
}

fn repl() -> Result<(), Box<dyn Error>> {
    println!("{}", Colour::Yellow.paint(format!("Fluid v{}", VERSION)));
    println!("{}", Colour::Green.paint("Type help for more information."));

    // Init repl editor
    let mut rl = Editor::<()>::new();
    rl.load_history("./history.txt").unwrap_or(());

    // Create codegen context
    let mut codegen = CodeGen::new("__repl__", CodeGenType::Repl);

    loop {
        let readline = rl.readline(">>> ");

        match readline {
            Ok(code) => {
                if code.starts_with(".") {
                    let command = &code.as_str()[1..];

                    match command {
                        "reset" => codegen.reset(),
                        _ => println!("{}: Invalid repl command `{}`", Colour::Red.bold().paint("error"), command),
                    }
                } else {
                    match code.as_str() {
                        "help" => println!("{}", Colour::Yellow.paint(HELP)),
                        _ => {
                            let mut lexer = Lexer::new(&code, &"<stdin>".into());
                            let tokens = match lexer.run() {
                                Ok(tokens) => tokens,
                                Err(errors) => {
                                    for err in errors {
                                        println!("{}", err);
                                    }

                                    continue;
                                }
                            };

                            let parser = Parser::new(tokens);

                            codegen.run(parser);
                        }
                    }
                }

                rl.add_history_entry(&code);
            }
            _ => break,
        }
    }

    codegen.free();

    // Save the editor histroy.
    rl.save_history("./history.txt")?;

    Ok(())
}
