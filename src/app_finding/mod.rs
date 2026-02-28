use std::path::Path;
use std::time::Instant;

use crate::{
    app::apps::SimpleApp,
    config::Config,
    utils::{get_config_file_path, read_config_file},
};
use rayon::prelude::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

/// Recursively loads apps from a set of folders.
///
/// [`exclude_patterns`] is a set of glob patterns to include, while [`include_patterns`] is a set of
/// patterns to include ignoring [`exclude_patterns`].
fn search_dir(
    path: impl AsRef<Path>,
    exclude_patterns: &[glob::Pattern],
    include_patterns: &[glob::Pattern],
    max_depth: usize,
) -> impl ParallelIterator<Item = SimpleApp> {
    use walkdir::WalkDir;

    WalkDir::new(path.as_ref())
        .follow_links(false)
        .max_depth(max_depth)
        .into_iter()
        .par_bridge()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "exe"))
        .filter_map(|entry| {
            let path = entry.path();

            if exclude_patterns.iter().any(|x| x.matches_path(path))
                && !include_patterns.iter().any(|x| x.matches_path(path))
            {
                #[cfg(debug_assertions)]
                tracing::trace!(
                    target: "dir_app_search",
                    "App excluded: {:?}", path.to_str()
                );

                return None;
            }

            let file_name = path.file_name().unwrap().to_string_lossy();
            let name = file_name.replace(".exe", "");

            #[cfg(debug_assertions)]
            tracing::trace!(
                target: "dir_app_search",
                "App added: {:?}", path.to_str()
            );

            #[cfg(target_os = "windows")]
            let icon = {
                use crate::cross_platform::windows::appicon::get_first_icon;

                get_first_icon(path)
                    .inspect_err(|e| {
                        tracing::error!("Error getting icon for {}: {e}", path.display());
                    })
                    .ok()
                    .flatten()
            };

            #[cfg(not(target_os = "windows"))]
            let icon = None;

            Some(SimpleApp::new_executable(
                &name,
                &name.to_lowercase(),
                "Application",
                path,
                icon,
            ))
        })
}

pub fn index_installed_apps(config: &Config) -> anyhow::Result<Vec<SimpleApp>> {
    tracing::debug!("Indexing installed apps");
    tracing::debug!("Exclude patterns: {:?}", &config.index_exclude_patterns);
    tracing::debug!("Include patterns: {:?}", &config.index_include_patterns);

    let path = get_config_file_path();
    let config = read_config_file(path.as_path())?;

    if config.index_dirs.is_empty() {
        tracing::debug!("No extra index dirs provided");
    }

    #[cfg(target_os = "windows")]
    {
        use windows::get_apps_from_registry;
        use windows::index_start_menu;

        let start = Instant::now();

        let mut other_apps = index_start_menu();
        get_apps_from_registry(&mut other_apps);

        let res = config
            .index_dirs
            .par_iter()
            .flat_map(|x| {
                search_dir(
                    &x.path,
                    &config.index_exclude_patterns,
                    &config.index_include_patterns,
                    x.max_depth,
                )
            })
            .chain(other_apps.into_par_iter())
            .collect();

        let end = Instant::now();
        tracing::info!(
            "Finished indexing apps (t = {}s)",
            (end - start).as_secs_f32()
        );

        Ok(res)
    }

    #[cfg(target_os = "macos")]
    {
        let start = Instant::now();

        let mut res: Vec<_> = config
            .index_dirs
            .par_iter()
            .flat_map(|x| {
                search_dir(
                    &x.path,
                    &config.index_exclude_patterns,
                    &config.index_include_patterns,
                    x.max_depth,
                )
            })
            .collect();

        res.extend(macos::get_installed_macos_apps(&config)?);

        let end = Instant::now();
        tracing::info!(
            "Finished indexing apps (t = {}s)",
            (end - start).as_secs_f32()
        );

        Ok(res)
    }

    #[cfg(target_os = "linux")]
    {
        let start = Instant::now();

        let other_apps = linux::get_installed_linux_apps(&config);

        let start2 = Instant::now();

        let res = config
            .index_dirs
            .par_iter()
            .flat_map(|x| {
                search_dir(
                    &x.path,
                    &config.index_exclude_patterns,
                    &config.index_include_patterns,
                    x.max_depth,
                )
            })
            .chain(other_apps.into_par_iter())
            .collect();

        let end = Instant::now();
        tracing::info!(
            "Finished indexing apps (t = {}s) (t2 = {}s)",
            (end - start).as_secs_f32(),
            (end - start2).as_secs_f32(),
        );

        Ok(res)
    }
}
