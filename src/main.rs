use clap::Parser;
use python_package_manager::{
    delete_package, install_from_requirements, install_from_requirements_parallel,
    install_packages, install_packages_parallel, list_packages, load_packages, save_packages,
    update_package, Cli, Commands,
};

fn main() {
    let args = Cli::parse();
    let mut package_registry = load_packages();

    match args.command {
        Commands::Install { packages, parallel } => {
            if packages.len() == 1 && packages[0].starts_with("-r=") {
                let requirements_path = &packages[0][3..];
                if parallel {
                    install_from_requirements_parallel(requirements_path, &mut package_registry);
                } else {
                    install_from_requirements(requirements_path, &mut package_registry);
                }
            } else if parallel {
                install_packages_parallel(&packages, &mut package_registry);
            } else {
                install_packages(&packages, &mut package_registry);
            }
        }
        Commands::Delete { name } => {
            delete_package(&name, &mut package_registry);
        }
        Commands::Update { name, version } => {
            update_package(&name, &version, &mut package_registry);
        }
        Commands::List => {
            list_packages(&package_registry);
        }
    }

    save_packages(&package_registry);
}
