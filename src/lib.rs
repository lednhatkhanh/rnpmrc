use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::os::unix;
use std::path::{Path, PathBuf};
use std::process::Command;

extern crate dirs;

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

    pub fn get_file_path(&self, config_dir: &Path) -> Result<PathBuf, &'static str> {
        let file_name = match &self.file_name {
            Some(file_name) => file_name,
            None => return Err("Didn't get a file name"),
        };

        let mut path_buf = config_dir.to_path_buf();

        path_buf.push(format!(".npmrc.{}", file_name));

        Ok(path_buf)
    }
}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    let mut config_dir = match dirs::home_dir() {
        Some(path_buffer) => path_buffer,
        None => return Err(Box::from("Error looking for home dir")),
    };

    config_dir.push(".rnpmrc");

    match config.command.as_str() {
        "create" => create_file(&config, &config_dir)?,
        "open" => open_file(&config, &config_dir)?,
        "symlink" => symlink_file(&config, Path::new(".npmrc"), &config_dir)?,
        "list" => list_files(&config_dir)?,
        "remove" => remove_file(&config, &config_dir)?,
        _ => return Err(Box::from("Unknown command")),
    };

    Ok(())
}

fn create_dir(config_dir: &Path) -> Result<(), io::Error> {
    let dir_exists = config_dir.is_dir();

    if !dir_exists {
        print!("Creating directory {}", config_dir.display());
        fs::DirBuilder::new().recursive(true).create(config_dir)?;
        println!("\nSucceed");
    }

    Ok(())
}

fn create_file(config: &Config, config_dir: &Path) -> Result<(), Box<dyn Error>> {
    create_dir(config_dir)?;

    let file_path = config.get_file_path(config_dir)?;
    let file_exists = file_path.exists();

    if file_exists {
        Err(Box::from(format!("{:?} exists", file_path)))
    } else {
        println!("Creating {:?}", file_path);
        fs::File::create(file_path)?;
        println!("Succeed");

        Ok(())
    }
}

fn open_file(config: &Config, config_dir: &Path) -> Result<(), Box<dyn Error>> {
    create_dir(config_dir)?;

    let file_path = config.get_file_path(config_dir)?;
    let file_exists = file_path.is_file();
    let process_name = match &config.editor {
        Some(editor_name) => editor_name.as_str(),
        None => "vim",
    };

    if file_exists {
        Command::new(process_name).arg(&file_path).status()?;
        Ok(())
    } else {
        Err(Box::from(format!("{:?} not found", file_path)))
    }
}

fn symlink_file(config: &Config, dest: &Path, config_dir: &Path) -> Result<(), Box<dyn Error>> {
    let file_path = config.get_file_path(config_dir)?;
    let file_exists = file_path.is_file();
    let npmrc_file_exists = Path::new(".npmrc").is_file();

    if file_exists {
        if npmrc_file_exists {
            println!("Removing .npmrc...");
            fs::remove_file(dest)?;
            println!("Succeed");
        }

        println!("Creating symlink for {:?}", file_path);
        unix::fs::symlink(file_path, dest)?;
        println!("Succeed");

        Ok(())
    } else {
        Err(Box::from(format!("{:?} not found", file_path)))
    }
}

fn list_files(config_dir: &Path) -> Result<(), Box<dyn Error>> {
    let paths = fs::read_dir(config_dir)?;
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

fn remove_file(config: &Config, config_dir: &Path) -> Result<(), Box<dyn Error>> {
    let file_path = config.get_file_path(config_dir)?;
    let file_exists = Path::new(&file_path).is_file();

    if file_exists {
        println!("Removing {:?}...", file_path);
        fs::remove_file(file_path)?;
        println!("Succeed");
        Ok(())
    } else {
        Err(Box::from(format!("{:?} doesn't exist", file_path)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPaths {
        profile_name: String,
        file_path: PathBuf,
        root_path: PathBuf,
        dir_path: PathBuf,
        npmrc_path: PathBuf,
    }

    fn get_test_configs() -> TestPaths {
        TestPaths {
            profile_name: String::from("test"),
            file_path: PathBuf::from(".test/.rnpmrc/.npmrc.test"),
            root_path: PathBuf::from(".test"),
            dir_path: PathBuf::from(".test/.rnpmrc"),
            npmrc_path: PathBuf::from(".test/.npmrc")
        }
    }

    fn remove_test_dir() -> Result<(), io::Error> {
        let root_path = get_test_configs().root_path;
        if root_path.is_dir() {
            fs::remove_dir_all(&root_path)?;
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

        let configs = get_test_configs();
        assert_eq!(create_dir(&configs.dir_path).is_ok(), true);
        assert_eq!(configs.dir_path.is_dir(), true);

        assert_eq!(after_each().is_ok(), true);
    }

    #[test]
    fn create_dir_ignores_when_exists() {
        assert_eq!(before_each().is_ok(), true);

        let configs = get_test_configs();
        assert_eq!(create_dir(&configs.dir_path).is_ok(), true);
        assert_eq!(configs.dir_path.is_dir(), true);

        assert_eq!(create_dir(&configs.dir_path).is_ok(), true);
        assert_eq!(configs.dir_path.is_dir(), true);

        assert_eq!(after_each().is_ok(), true);
    }

    #[test]
    fn create_file_success() {
        assert_eq!(before_each().is_ok(), true);

        let test_configs = get_test_configs();
        let config = Config {
            file_name: Some(test_configs.profile_name.clone()),
            command: String::from("create"),
            editor: None,
        };

        assert_eq!(create_file(&config, &test_configs.dir_path).is_ok(), true);
        assert_eq!(test_configs.file_path.is_file(), true);

        assert_eq!(after_each().is_ok(), true);
    }

    #[test]
    fn create_file_error_when_file_exists() {
        assert_eq!(before_each().is_ok(), true);

        let test_configs = get_test_configs();
        let config = Config {
            file_name: Some(test_configs.profile_name.clone()),
            command: String::from("create"),
            editor: None,
        };

        assert_eq!(create_file(&config, &test_configs.dir_path).is_ok(), true);
        assert_eq!(test_configs.file_path.is_file(), true);

        assert_eq!(create_file(&config, &test_configs.dir_path).is_err(), true);

        assert_eq!(after_each().is_ok(), true);
    }

    #[test]
    fn remove_file_success() {
        assert_eq!(before_each().is_ok(), true);

        let test_configs = get_test_configs();
        let config = Config {
            file_name: Some(test_configs.profile_name.clone()),
            command: String::from("create"),
            editor: None,
        };

        assert_eq!(create_file(&config, &test_configs.dir_path).is_ok(), true);

        assert_eq!(remove_file(&config, &test_configs.dir_path).is_ok(), true);
        assert_eq!(test_configs.file_path.exists(), false);

        assert_eq!(after_each().is_ok(), true);
    }

    #[test]
    fn remove_file_error_when_file_does_not_exist() {
        assert_eq!(before_each().is_ok(), true);

        let test_configs = get_test_configs();
        let config = Config {
            file_name: Some(test_configs.profile_name.clone()),
            command: String::from("create"),
            editor: None,
        };

        assert_eq!(remove_file(&config, &test_configs.dir_path).is_err(), true);

        assert_eq!(after_each().is_ok(), true);
    }

    #[test]
    fn symlink_file_success() {
        assert_eq!(before_each().is_ok(), true);

        let test_configs = get_test_configs();
        let config = Config {
            file_name: Some(test_configs.profile_name.clone()),
            command: String::from("create"),
            editor: None,
        };

        assert_eq!(create_file(&config, &test_configs.dir_path).is_ok(), true);

        assert_eq!(symlink_file(&config, &test_configs.npmrc_path, &test_configs.dir_path).is_ok(), true);
        assert_eq!(fs::read_link(&test_configs.npmrc_path).is_ok(), true);

        assert_eq!(after_each().is_ok(), true);
    }

    #[test]
    fn symlink_file_error_when_file_doesnot_exist() {
        assert_eq!(before_each().is_ok(), true);

        let test_configs = get_test_configs();
        let config = Config {
            file_name: Some(test_configs.profile_name.clone()),
            command: String::from("create"),
            editor: None,
        };

        assert_eq!(symlink_file(&config, &test_configs.npmrc_path, &test_configs.dir_path).is_err(), true);
        assert_eq!(fs::read_link(&test_configs.npmrc_path).is_ok(), false);

        assert_eq!(after_each().is_ok(), true);
    }
}
