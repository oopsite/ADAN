use std::io::{self, Write};
use clearscreen;
use std::fs;
use std::path::{Path};

pub enum Terminal {
    Output,             // Log a message to the terminal. Must be one concatenated string literal.
    Input,              // Listens and return a user's input. Stops listening when `Enter` pressed.
    Clear,              // Simulates a terminal clearing so you can cleanly see any upcoming
                        // messages.
}

impl Terminal {
    pub fn output(input: &str, newline: bool) {
        if newline {
            println!("{input}");
        } else {
            print!("{input}");
        }
    }

    pub fn input() -> String {
        let mut output = String::new();
    
        io::stdin().read_line(&mut output).expect("Failed to read user input.");
        output.trim_end().to_string()
    }

    pub fn clear() {
        clearscreen::clear().expect("Failed to clear the screen");
    }
}


pub enum File {
    Read,      // Intakes a file directory then outputs the contents of such file.
    Write,     // Creates a file, if not already existant, then overrides existing contents to x into y file.
    Append,    // Creates a file, if not already existant, then appends x content.
    Clone,     // Duplicates x file into the same directory.
    Delete,    // Removes x file from the specified directory, optionally recursive.
    Exists,    // Returns a boolean depending if the file specified exists.
    Rename,
}

impl File {
    pub fn read(file_path: &str) -> io::Result<String> {
        fs::read_to_string(file_path)
    }

    pub fn write(file_path: &str, contents: &str) -> io::Result<()> {
        fs::write(file_path, contents)
    }

    pub fn append(file_path: &str, contents: &str) -> io::Result<()> {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(file_path)?;
    
        writeln!(file, "{}", contents)?;
        Ok(())
    }

    pub fn clone(file_path: &str) -> io::Result<()> {
        let contents = Self::read(file_path)?;
        let new_path = format!("{}_clone", file_path);
     
        Self::write(&new_path, &contents)
    }

    pub fn delete(file_path: &str) -> io::Result<()> {
        fs::remove_file(file_path)
    }

    pub fn exists(file_path: &str) -> bool {
        fs::exists(file_path).is_err()
    }

    pub fn rename(file_path: &str, new_name: &str) -> io::Result<()> {
        fs::rename(file_path, new_name)
    }
}


enum Folder {
    Children,   // Lists how many first-generation sub-folders/files are in the directory.
    Clone,      // Creates a 1:1 clone of x folder.
    Delete,     // Attempts to remove x folder from y directory.
    Rename,
    Exists,
}

impl Folder {
    pub fn children(folder_path: &str) -> io::Result<usize> {
        let entries = fs::read_dir(folder_path)?;
        
        Ok(entries.count())
    }

    pub fn clone(folder_path: &str) -> io::Result<()> {
        let src = Path::new(folder_path);
        if !src.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Folder not found"));
        }

        let new_path = format!("{}_clone", folder_path);
        fs::create_dir_all(&new_path)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let dest_path = Path::new(&new_path).join(entry.file_name());

            if path.is_dir() {
                Folder::clone(path.to_str().unwrap())?;
            } else {
                fs::copy(&path, &dest_path)?;
            }
        }

        Ok(())
    }

    pub fn delete(folder_path: &str) -> io::Result<()> {
        fs::remove_dir_all(folder_path)
    }

    pub fn rename(folder_path: &str, new_name: &str) -> io::Result<()> {
        fs::rename(folder_path, new_name)
    }

    pub fn exists(folder_path: &str) -> bool {
        Path::new(folder_path).exists()
    }
}
