## Writing Simplified

Journal is a terminal based writing tool, developed for writers who want a distraction free experience, and the ability to set up their writing environment as fast as possible.

The application is entirely terminal based and uses the Vim text editor. 

To use, please change the filepath of the ENTRY_DIR variable in the main.rs file, to the path you'd like to store your work. The project is currently made solely for my own use but this could change in the future.

### Requirements

Rust + Cargo, sqlite and vim.

Build the project using 

`cargo build --release`

and move the compiled binary to your computers path.

This project is designed with Linux Systems in mind, but should work on MacOS and Windows
