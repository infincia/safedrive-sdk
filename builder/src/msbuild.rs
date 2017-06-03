use subprocess::{Exec, ExitStatus};
use platform::Platform;
use error::BuildError;
use configuration::Configuration;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub struct MSBuild {
    solution: String,
    configuration: Configuration,
    platform: Platform,
    build_prefix: PathBuf,
}

impl MSBuild {
    pub fn new(solution: &str,
               configuration: Configuration,
               platform: Platform,
               build_prefix: &Path) -> MSBuild {
        MSBuild {
            solution: solution.to_string(),
            configuration: configuration,
            platform: platform,
            build_prefix: build_prefix.to_owned(),
        }
    }

    pub fn build(&self) -> Result<(), BuildError> {
        info!("running msbuild for: {}", &self.solution);

        let exit = Exec::cmd("msbuild")
            .arg("/m")
            .arg("/v:n")
            //.arg(format!("/p:OutDir={}\\", self.build_prefix.display()))
            .arg("/p:WholeProgramOptimization=false")
            .arg(format!("/p:RuntimeLibrary={}", self.configuration.runtime_library().name()))
            .arg(format!("/p:Configuration={}", self.configuration.name()))

            .arg(format!("/p:Platform={}", self.platform.as_ref()))

            .arg(format!("/p:PlatformToolset={}", self.configuration.toolset().name()))
            .arg(format!("{}.sln", &self.solution))
            .join()?;

        match exit {
            ExitStatus::Exited(0) => {
                info!("msbuild exited cleanly");

            },
            _ => {
                warn!("msbuild exited with failure");

                return Err(BuildError::BuildFailed(format!("{} {:?}", self.solution, exit)));
            }
        }

        Ok(())
    }
}

