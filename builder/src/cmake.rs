use subprocess::{Exec, ExitStatus};
use platform::Platform;
use error::BuildError;
use configuration::Configuration;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub struct CMake {
    solution: String,
    configuration: Configuration,
    platform: Platform,
    build_prefix: PathBuf,
}

impl CMake {
    pub fn new(solution: &str,
               configuration: Configuration,
               platform: Platform,
               build_prefix: &Path) -> CMake {
        CMake {
            solution: solution.to_string(),
            configuration: configuration,
            platform: platform,
            build_prefix: build_prefix.to_owned(),
        }
    }

    pub fn build(&self) -> Result<(), BuildError> {
        info!("running cmake for: {}", &self.solution);

        let cmake_gen = match self.platform {
            Platform::i686 => {
                format!("-G{}", self.configuration.toolset().vs_version())
            },
            Platform::x86_64 => {
                format!("-G{} Win64", self.configuration.toolset().vs_version())
            }
        };

        let exit = Exec::cmd("cmake")
            .arg(".")
            .arg(cmake_gen)
            .arg(format!("-T{}", self.configuration.toolset().name()))
            .arg("-DBUILD_SHARED_LIBS=0")
            .arg("-DBUILD_EXAMPLES=0")
            .arg("-DBUILD_TESTING=0")
            //.arg(format!("-DCMAKE_LIBRARY_OUTPUT_DIRECTORY={}", &self.build_prefix.display()))
            //.arg(format!("-DCMAKE_ARCHIVE_OUTPUT_DIRECTORY={}", &self.build_prefix.display()))
            .arg(format!("-DOPENSSL_USE_STATIC_LIBS=TRUE"))
            .arg(format!("-DCRYPTO_BACKEND=WinCNG"))
            .arg(format!("-DOPENSSL_ROOT_DIR={}", &self.build_prefix.display()))
            .arg(format!("-DOPENSSL_INCLUDE_DIR={}\\include", &self.build_prefix.display()))
            .arg(format!("-DCMAKE_BUILD_TYPE={}", self.configuration.name()))
            .arg(format!("-DCMAKE_C_FLAGS_RELEASE={}", self.configuration.runtime_library().c_flags_release()))
            .arg(format!("-DCMAKE_CXX_FLAGS_RELEASE={}", self.configuration.runtime_library().cxx_flags_release()))
            .arg(format!("-DCMAKE_C_FLAGS_DEBUG={}", self.configuration.runtime_library().c_flags_debug()))
            .arg(format!("-DCMAKE_CXX_FLAGS_DEBUG={}", self.configuration.runtime_library().cxx_flags_debug()))
            .join()?;

        match exit {
            ExitStatus::Exited(0) => {
                info!("cmake exited cleanly");
            },
            _ => {
                warn!("cmake exited with failure");

                return Err(BuildError::BuildFailed(format!("{} {:?}", self.solution, exit)));
            }
        }

        Ok(())
    }
}

