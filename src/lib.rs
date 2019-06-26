use clap::ArgMatches;
use failure::ResultExt;
use std::fs;
use std::os::unix;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run(matches: ArgMatches) -> Result<(), failure::Error> {
    let (home_dir, config_dir) = get_config_path()?;

    create_config_dir(&config_dir).with_context(|_| "failed to create config dir".to_string())?;
    create_config_file(&config_dir).with_context(|_| "failed to create config file".to_string())?;

    match matches.subcommand() {
        ("create", Some(create_matches)) => {
            let profile = create_matches.value_of("profile").unwrap();

            create_profile(profile, &config_dir)
                .with_context(|_| format!("Failed to create profile \"{}\"", profile))?;
        }
        ("list", Some(_)) => {
            list_all_profiles(&config_dir).with_context(|_| "Failed to list all profiles")?;
        }
        ("open", Some(open_matcher)) => {
            let profile = open_matcher.value_of("profile").unwrap();
            let editor = open_matcher.value_of("editor").unwrap();

            open_profile(profile, &config_dir, editor)
                .with_context(|_| format!("Failed to open profile \"{}\"", profile))?;
        }
        ("activate", Some(activate_matcher)) => {
            let profile = activate_matcher.value_of("profile").unwrap();

            activate_profile(profile, &config_dir, &home_dir)
                .with_context(|_| format!("Failed to activate profile \"{}\"", profile))?;
        }
        ("status", Some(_)) => {
            show_active_profile(&config_dir, &home_dir);
        }
        ("remove", Some(remove_matches)) => {
            let profile = remove_matches.value_of("profile").unwrap();

            remove_profile(profile, &config_dir)
                .with_context(|_| format!("Failed to remove profile \"{}\"", profile))?;
        }
        ("", None) => eprintln!("No subcommand was used"),
        _ => unreachable!(),
    };

    Ok(())
}

fn get_config_path() -> Result<(PathBuf, PathBuf), failure::Error> {
    let home_dir = match dirs::home_dir() {
        Some(path) => path,
        None => return Err(failure::err_msg("did not find home directory")),
    };

    let mut config_dir = PathBuf::from(&home_dir);
    config_dir.push(".rnpmrc");

    Ok((home_dir, config_dir))
}

fn update_config_file(content: &str, config_dir: &Path) -> Result<(), failure::Error> {
    let config_file = build_file_path(config_dir, ".config");

    if !config_file.is_file() {
        return Err(failure::err_msg(format!(
            "file {:?} doesn't exists",
            config_file
        )));
    }

    fs::write(&config_file, content)?;

    Ok(())
}

pub fn create_config_file(config_dir: &Path) -> Result<(), failure::Error> {
    let file_path = build_file_path(config_dir, ".config");

    if !file_path.is_file() {
        print!("Creating file {:?}... ", file_path);
        fs::File::create(file_path)?;
        println!("Succeed");
    }

    Ok(())
}

fn create_config_dir(dir_path: &Path) -> Result<(), failure::Error> {
    let dir_exists = dir_path.is_dir();

    if !dir_exists {
        print!("Creating directory {:?}... ", dir_path);
        fs::DirBuilder::new().recursive(true).create(dir_path)?;
        println!("Succeed");
    }

    Ok(())
}

fn create_profile(profile: &str, config_dir: &Path) -> Result<(), failure::Error> {
    let file_path = build_file_path(config_dir, &format!(".npmrc.{}", profile));

    if file_path.is_file() {
        Err(failure::err_msg(format!(
            "file {:?} already exists",
            file_path
        )))
    } else {
        print!("Creating file {:?}... ", file_path);
        fs::File::create(file_path)?;
        println!("Succeed");

        Ok(())
    }
}

fn list_all_profiles(config_dir: &Path) -> Result<(), failure::Error> {
    let paths = fs::read_dir(config_dir)?;
    let mut file_names = String::new();

    for entry in paths {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let file_name = match path.file_name() {
                Some(parsed_file_name) => parsed_file_name,
                None => return Err(failure::err_msg("Failed reading files".to_string())),
            };

            if let Some(file_name_str) = file_name.to_str() {
                if file_name_str.contains(".npmrc.") {
                    file_names.push_str(&format!("{}\n", file_name_str));
                }
            }
        }
    }

    println!("{}", file_names);

    Ok(())
}

fn open_profile(profile: &str, config_dir: &Path, editor: &str) -> Result<(), failure::Error> {
    let file_path = build_file_path(config_dir, &format!(".npmrc.{}", profile));

    if file_path.is_file() {
        Command::new(editor).arg(file_path).status()?;
        Ok(())
    } else {
        Err(failure::err_msg(format!(
            "file {:?} doesn't exists",
            file_path
        )))
    }
}

fn remove_profile(profile: &str, config_dir: &Path) -> Result<(), failure::Error> {
    let file_path = build_file_path(config_dir, &format!(".npmrc.{}", profile));

    if file_path.is_file() {
        print!("Removing file {:?}... ", file_path);
        fs::remove_file(file_path)?;
        println!("Succeed");

        update_config_file("", &config_dir)?;

        Ok(())
    } else {
        Err(failure::err_msg(format!(
            "file {:?} doesn't exists",
            file_path
        )))
    }
}

fn activate_profile(
    profile: &str,
    config_dir: &Path,
    home_dir: &Path,
) -> Result<(), failure::Error> {
    let file_path = build_file_path(config_dir, &format!(".npmrc.{}", profile));
    let npmrc_path = build_file_path(home_dir, ".npmrc");

    if !file_path.is_file() {
        return Err(failure::err_msg(format!(
            "file {:?} doesn't exists",
            file_path
        )));
    }

    if exists_or_symlinked(&npmrc_path) {
        print!("Removing {:?}... ", npmrc_path);
        fs::remove_file(&npmrc_path)?;
        println!("Succeed");
    }

    print!("Creating symlink for {:?}... ", file_path);
    unix::fs::symlink(&file_path, &npmrc_path)?;
    println!("Succeed");

    update_config_file(&format!("active-profile={}", profile), &config_dir)?;

    Ok(())
}

fn show_active_profile(config_dir: &Path, home_dir: &Path) {
    let npmrc_path = build_file_path(home_dir, ".npmrc");

    match fs::read_link(&npmrc_path) {
        Ok(info) => {
            if info.is_file() && info.starts_with(&config_dir) {
                if let Some(file_name) = info.file_name() {
                    println!("{:?} is active", file_name);
                } else {
                    println!("No active profile");
                }
            } else {
                println!("No active profile");
            }
        }
        Err(_) => {
            println!("No active profile");
        }
    };
}

// Utilities
fn build_file_path(dir_path: &Path, file_name: &str) -> PathBuf {
    let mut file_path = PathBuf::from(dir_path);
    file_path.push(file_name);

    file_path
}

fn exists_or_symlinked(path: &Path) -> bool {
    if path.is_file() || path.is_dir() {
        return true;
    }

    if fs::read_link(&path).is_ok() {
        return true;
    }

    false
}
