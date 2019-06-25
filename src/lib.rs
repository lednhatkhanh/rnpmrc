use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::os::unix;
use std::path::Path;

extern crate subprocess;

pub struct Config {
    file_name: Option<String>,
    command: String,
    editor: Option<String>,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, &'static str> {
        args.next();

        let command = match args.next() {
            Some(args) => args,
            None => return Err("Didn't get a command"),
        };

        let profile_name = args.next();
        let editor = args.next();

        Ok(Config {
            command,
            file_name: profile_name,
            editor,
        })
    }

    pub fn get_full_path(&self, base_path: &str) -> Result<String, &'static str> {
        match &self.file_name {
            Some(file_name) => Ok(format!("{}/{}", base_path, file_name)),
            None => Err("Didn't get a file name"),
        }
    }
}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    const PATH: &str = ".rnpm";

    match config.command.as_str() {
        "create" => create_file(&config, PATH)?,
        "open" => open_file(&config, PATH)?,
        "symlink" => symlink_file(&config, PATH)?,
        "list" => list_files(PATH)?,
        "remove" => remove_file(&config, PATH)?,
        _ => return Err(Box::from("Unknown command")),
    };

    Ok(())
}

fn create_dir(path: &str) -> Result<(), io::Error> {
    let dir_exists = Path::new(path).is_dir();

    if !dir_exists {
        println!("Creating directory {}...", path);
        fs::DirBuilder::new().create(path)?;
        println!("Succeed");
    }

    Ok(())
}

fn create_file(config: &Config, base_path: &str) -> Result<(), Box<dyn Error>> {
    create_dir(base_path)?;

    let full_path = config.get_full_path(base_path)?;
    let file_exists = Path::new(&full_path).is_file();

    if file_exists {
        println!("{} exists", full_path);
    } else {
        println!("Creating {}...", full_path);
        fs::File::create(&full_path)?;
        println!("Succeed");
    }

    Ok(())
}

fn open_file(config: &Config, base_path: &str) -> Result<(), Box<dyn Error>> {
    create_dir(base_path)?;

    let full_path = config.get_full_path(base_path)?;
    let file_exists = Path::new(&full_path).is_file();
    let process_name = match &config.editor {
        Some(editor_name) => &editor_name[..],
        None => "vim",
    };

    if file_exists {
        subprocess::Exec::cmd(process_name).arg(&full_path).join()?;
        Ok(())
    } else {
        Err(Box::from(format!("{} not found", full_path)))
    }
}

fn symlink_file(config: &Config, base_path: &str) -> Result<(), Box<dyn Error>> {
    let full_path = config.get_full_path(base_path)?;
    let file_exists = Path::new(&full_path).is_file();
    let npmrc_file_exists = Path::new(".npmrc").is_file();

    if file_exists {
        if npmrc_file_exists {
            println!("Removing .npmrc...");
            fs::remove_file(".npmrc")?;
            println!("Succeed");
        }

        println!("Creating symlink for {}", full_path);
        unix::fs::symlink(full_path, ".npmrc")?;
        println!("Succeed");

        Ok(())
    } else {
        Err(Box::from(format!("{} not found", full_path)))
    }
}

fn list_files(base_path: &str) -> Result<(), Box<dyn Error>> {
    let paths = fs::read_dir(base_path)?;
    let mut file_names = String::new();

    for entry in paths {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let file_name = match path.file_name() {
                Some(parsed_file_name) => match parsed_file_name.to_str() {
                    Some(sliced_file_name) => sliced_file_name,
                    None => return Err(Box::from("Error parsing file")),
                },
                None => return Err(Box::from("Error parsing file")),
            };

            file_names.push_str(&format!("{}\n", file_name))
        }
    }

    println!("{}", file_names);

    Ok(())
}

fn remove_file(config: &Config, base_path: &str) -> Result<(), Box<dyn Error>> {
    let full_path = config.get_full_path(base_path)?;
    let file_exists = Path::new(&full_path).is_file();

    if file_exists {
        println!("Removing {}...", full_path);
        fs::remove_file(full_path)?;
        println!("Succeed");
        Ok(())
    } else {
        Err(Box::from(format!("{} doesn't exist", full_path)))
    }
}
