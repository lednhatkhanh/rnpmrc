use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::os::unix;
use std::path::Path;
use std::process::Command;

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
        "symlink" => symlink_file(&config, ".npmrc", PATH)?,
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
        fs::DirBuilder::new().recursive(true).create(path)?;
        println!("Succeed");
    }

    Ok(())
}

fn create_file(config: &Config, base_path: &str) -> Result<(), Box<dyn Error>> {
    create_dir(base_path)?;

    let full_path = config.get_full_path(base_path)?;
    let file_exists = Path::new(&full_path).is_file();

    if file_exists {
        Err(Box::from(format!("{} exists", full_path)))
    } else {
        println!("Creating {}...", full_path);
        fs::File::create(&full_path)?;
        println!("Succeed");

        Ok(())
    }
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
        Command::new(process_name).arg(&full_path).status()?;
        Ok(())
    } else {
        Err(Box::from(format!("{} not found", full_path)))
    }
}

fn symlink_file(config: &Config, dest: &str, base_path: &str) -> Result<(), Box<dyn Error>> {
    let full_path = config.get_full_path(base_path)?;
    let file_exists = Path::new(&full_path).is_file();
    let npmrc_file_exists = Path::new(".npmrc").is_file();

    if file_exists {
        if npmrc_file_exists {
            println!("Removing .npmrc...");
            fs::remove_file(dest)?;
            println!("Succeed");
        }

        println!("Creating symlink for {}", full_path);
        unix::fs::symlink(full_path, dest)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT_PATH: &str = ".test";
    const NPMRC_PATH: &str = ".test/.npmrc";
    const PATH: &str = ".test/.rnpm";

    fn remove_test_dir() -> Result<(), io::Error> {
        if Path::new(ROOT_PATH).is_dir() {
            fs::remove_dir_all(ROOT_PATH)?;
        }

        Ok(())
    }

    fn before_each() -> Result<(), io::Error> {
        remove_test_dir()
    }

    fn after_each() -> Result<(), io::Error> {
        remove_test_dir()
    }

    #[test]
    fn create_dir_success() {
        assert_eq!(before_each().is_ok(), true);

        assert_eq!(create_dir(PATH).is_ok(), true);
        assert_eq!(Path::new(PATH).is_dir(), true);

        assert_eq!(after_each().is_ok(), true);
    }

    #[test]
    fn create_dir_ignores_when_exists() {
        assert_eq!(before_each().is_ok(), true);

        assert_eq!(create_dir(PATH).is_ok(), true);
        assert_eq!(Path::new(PATH).is_dir(), true);

        assert_eq!(create_dir(PATH).is_ok(), true);
        assert_eq!(Path::new(PATH).is_dir(), true);

        assert_eq!(after_each().is_ok(), true);
    }

    #[test]
    fn create_file_success() {
        assert_eq!(before_each().is_ok(), true);

        let config = Config {
            file_name: Some(String::from("test")),
            command: String::from("create"),
            editor: None,
        };

        assert_eq!(create_file(&config, PATH).is_ok(), true);
        assert_eq!(Path::new(&format!("{}/{}", PATH, "test")).is_file(), true);

        assert_eq!(after_each().is_ok(), true);
    }

    #[test]
    fn create_file_error_when_file_exists() {
        assert_eq!(before_each().is_ok(), true);

        let config = Config {
            file_name: Some(String::from("test")),
            command: String::from("create"),
            editor: None,
        };

        assert_eq!(create_file(&config, PATH).is_ok(), true);
        assert_eq!(Path::new(&format!("{}/{}", PATH, "test")).is_file(), true);

        assert_eq!(create_file(&config, PATH).is_err(), true);

        assert_eq!(after_each().is_ok(), true);
    }

    #[test]
    fn remove_file_success() {
        assert_eq!(before_each().is_ok(), true);

        let config = Config {
            file_name: Some(String::from("test")),
            command: String::from("create"),
            editor: None,
        };

        assert_eq!(create_file(&config, PATH).is_ok(), true);

        assert_eq!(remove_file(&config, PATH).is_ok(), true);
        assert_eq!(Path::new(&format!("{}/{}", PATH, "test")).exists(), false);

        assert_eq!(after_each().is_ok(), true);
    }

    #[test]
    fn remove_file_error_when_file_does_not_exist() {
        assert_eq!(before_each().is_ok(), true);

        let config = Config {
            file_name: Some(String::from("test")),
            command: String::from("create"),
            editor: None,
        };

        assert_eq!(remove_file(&config, PATH).is_err(), true);

        assert_eq!(after_each().is_ok(), true);
    }

    #[test]
    fn symlink_file_success() {
        assert_eq!(before_each().is_ok(), true);

        let config = Config {
            file_name: Some(String::from("test")),
            command: String::from("create"),
            editor: None,
        };

        assert_eq!(create_file(&config, PATH).is_ok(), true);

        assert_eq!(symlink_file(&config, NPMRC_PATH, PATH).is_ok(), true);
        assert_eq!(fs::read_link(NPMRC_PATH).is_ok(), true);

        assert_eq!(after_each().is_ok(), true);
    }

    #[test]
    fn symlink_file_error_when_file_doesnot_exist() {
        assert_eq!(before_each().is_ok(), true);

        let config = Config {
            file_name: Some(String::from("test")),
            command: String::from("create"),
            editor: None,
        };

        assert_eq!(symlink_file(&config, NPMRC_PATH, PATH).is_err(), true);
        assert_eq!(fs::read_link(NPMRC_PATH).is_ok(), false);

        assert_eq!(after_each().is_ok(), true);
    }
}
