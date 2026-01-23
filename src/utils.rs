//! This has all the utility functions that rustcast uses
use std::{
    io,
    path::{Path, PathBuf},
    thread,
};

#[cfg(target_os = "macos")]
use icns::IconFamily;
use rayon::prelude::*;

#[cfg(target_os = "macos")]
use {
    crate::cross_platform::macos::get_installed_macos_apps, objc2_app_kit::NSWorkspace,
    objc2_foundation::NSURL, std::os::unix::fs::PermissionsExt,
};

#[cfg(target_os = "windows")]
use std::process::Command;

use crate::app::apps::App;

pub fn get_config_installation_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        std::env::var("LOCALAPPDATA").unwrap().into()
    } else {
        std::env::var("HOME").unwrap().into()
    }
}

pub fn get_config_file_path() -> PathBuf {
    let home = get_config_installation_dir();

    if cfg!(target_os = "windows") {
        home.join("rustcast/config.toml")
    } else {
        home.join(".config/rustcast/config.toml")
    }
}

/// Recursively loads apps from a set of folders.
///
/// [`exclude_patterns`] is a set of glob patterns to include, while [`include_patterns`] is a set of
/// patterns to include ignoring [`exclude_patterns`].
fn search_dir(
    path: impl AsRef<Path>,
    exclude_patterns: &[glob::Pattern],
    include_patterns: &[glob::Pattern],
    max_depth: usize,
) -> impl ParallelIterator<Item = App> {
    use crate::{app::apps::AppCommand, commands::Function};
    use walkdir::WalkDir;

    WalkDir::new(path.as_ref())
        .follow_links(false)
        .max_depth(max_depth)
        .into_iter()
        .par_bridge()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "exe"))
        .filter_map(|entry| {
            let path = entry.path();

            if exclude_patterns.iter().any(|x| x.matches_path(path))
                && !include_patterns.iter().any(|x| x.matches_path(path))
            {
                #[cfg(debug_assertions)]
                tracing::trace!("Executable skipped [kfolder]: {:?}", path.to_str());

                return None;
            }

            let file_name = path.file_name().unwrap().to_string_lossy();
            let name = file_name.replace(".exe", "");

            #[cfg(debug_assertions)]
            tracing::trace!("Executable loaded  [kfolder]: {:?}", path.to_str());

            Some(App {
                open_command: AppCommand::Function(Function::OpenApp(
                    path.to_string_lossy().to_string(),
                )),
                name: name.clone(),
                name_lc: name.to_lowercase(),
                icons: None,
                desc: "Application".to_string(),
            })
        })
}

use crate::config::Config;

pub fn read_config_file(file_path: &Path) -> anyhow::Result<Config> {
    match std::fs::read_to_string(file_path) {
        Ok(a) => Ok(toml::from_str(&a)?),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let cfg = Config::default();
            std::fs::write(
                file_path,
                toml::to_string(&cfg).unwrap_or_else(|x| x.to_string()),
            )?;
            Ok(cfg)
        }
        Err(e) => Err(e.into()),
    }
}

pub fn open_application(path: &str) {
    let path_string = path.to_string();
    thread::spawn(move || {
        let path = &path_string;
        #[cfg(target_os = "windows")]
        {
            println!("Opening application: {}", path);

            Command::new("powershell")
                .arg(format!("Start-Process '{}'", path))
                .status()
                .ok();
        }

        #[cfg(target_os = "macos")]
        {
            NSWorkspace::new().openURL(&NSURL::fileURLWithPath(
                &objc2_foundation::NSString::from_str(path),
            ));
        }

        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open").arg(path).status().ok();
        }
    });
}

pub fn index_installed_apps(config: &Config) -> anyhow::Result<Vec<App>> {
    tracing::debug!("Indexing installed apps");
    tracing::debug!("Exclude patterns: {:?}", &config.index_exclude_patterns);
    tracing::debug!("Include patterns: {:?}", &config.index_include_patterns);

    let path = get_config_file_path();
    let config = read_config_file(path.as_path())?;

    if config.index_dirs.is_empty() {
        tracing::debug!("No extra index dirs provided")
    }

    #[cfg(target_os = "windows")]
    {
        use std::time::Instant;

        use crate::cross_platform::windows::app_finding::get_apps_from_registry;
        use crate::cross_platform::windows::app_finding::get_known_paths;

        let start = Instant::now();

        let mut reg_apps = Vec::new();
        get_apps_from_registry(&mut reg_apps);

        let res = config
            .index_dirs
            .par_iter()
            .chain(get_known_paths().par_iter())
            .flat_map(|x| {
                search_dir(
                    x,
                    &config.index_exclude_patterns,
                    &config.index_include_patterns,
                    3,
                )
            })
            .chain(reg_apps.into_par_iter())
            .collect();

        let end = Instant::now();
        tracing::info!(
            "Finished indexing apps (t = {}s)",
            (end - start).as_secs_f32()
        );

        Ok(res)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let start = Instant::now();

        let res = config
            .index_dirs
            .par_iter()
            .chain(get_known_paths().par_iter())
            .flat_map(|x| {
                search_dir(
                    x,
                    &config.index_exclude_patterns,
                    &config.index_exclude_patterns,
                    3,
                )
            })
            .collect();

        let end = Instant::now();
        tracing::info!(
            "Finished indexing apps (t = {}s)",
            (end - start).as_secs_f32()
        );

        Ok(res)
    }
}
