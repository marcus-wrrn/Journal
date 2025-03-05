## Writing Simplified

Journal is a terminal based writing tool, developed for writers who want a distraction free experience and fast setup. 

If you spend a lot of time in terminal environments like I do, Journal can be used to spin up a blank page in as little as a second. 

The application is entirely terminal based and uses the Vim text editor. 

To use, please change the filepath of the ENTRY_DIR variable in the main.rs file, to the path you'd like to store your work. The project is currently made solely for my own use but this could change in the future.

### Requirements

Rust + Cargo, sqlite and vim.

Build the project using 

`cargo build --release`

and move the compiled binary to your computers path.

To initialize the database run 
`./journal --init_tables`

Now you're good to start writing!

This project is designed with Linux Systems in mind, but should work on MacOS and Windows

