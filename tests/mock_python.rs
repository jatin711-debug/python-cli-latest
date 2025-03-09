use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

pub struct MockPython {
    temp_dir: TempDir,
    pub python_path: PathBuf,
}

impl MockPython {
    pub fn new() -> io::Result<Self> {
        let temp_dir = TempDir::new()?;
        let python_path = temp_dir.path().join(if cfg!(windows) { "python.exe" } else { "python" });
        
        // Create mock Python executable
        let mut file = File::create(&python_path)?;
        if cfg!(unix) {
            write!(file, "#!/bin/sh\n")?;
            write!(file, "echo 'Mock Python Environment'\n")?;
            write!(file, "exit 0\n")?;
        } else {
            write!(file, "@echo off\n")?;
            write!(file, "echo Mock Python Environment\n")?;
            write!(file, "exit /b 0\n")?;
        }
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&python_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&python_path, perms)?;
        }

        Ok(Self {
            temp_dir,
            python_path,
        })
    }

    pub fn with_pip_response(self, response: &str) -> io::Result<Self> {
        let pip_path = self.temp_dir.path().join("pip_responder.json");
        let mut file = File::create(&pip_path)?;
        write!(file, "{}", response)?;
        Ok(self)
    }

    pub fn add_to_path(&self) {
        let path_var = env::var_os("PATH").unwrap_or_default();
        let mut paths = env::split_paths(&path_var).collect::<Vec<_>>();
        paths.insert(0, self.temp_dir.path().to_path_buf());
        env::set_var("PATH", env::join_paths(paths).unwrap());
    }
}