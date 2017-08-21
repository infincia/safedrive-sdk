use subprocess::{Exec, ExitStatus};
use platform::Platform;
use error::BuildError;

pub struct VC {
    platform: Platform
}

impl VC {
    pub fn new(platform: Platform) -> VC {
        VC {
            platform: platform,
        }
    }

    pub fn load_escript(&self) -> Result<(), BuildError>  {
        info!("running escript for: {}", &self.platform.name());

        let exit = Exec::cmd(&self.platform.escript())
            .arg(&self.platform.name())
            .join()?;


        match exit {
            ExitStatus::Exited(0) => {
                info!("escript exited cleanly");

            },
            _ => {
                warn!("escript exited with failure");

                return Err(BuildError::BuildFailed(format!("{} {:?}", &self.platform.name(), exit)));
            }
        }

        Ok(())
    }

    pub fn load_env(&self) -> Result<(), BuildError>  {
        info!("running vcvars for: {}", &self.platform.name());

        let exit = Exec::cmd(r#"C:\Program Files (x86)\Microsoft Visual Studio\2017\Community\VC\Auxiliary\Build\vcvarsall.bat"#)
            .arg(&self.platform.name())
            .join()?;


        match exit {
            ExitStatus::Exited(0) => {
                info!("vcvars exited cleanly");

            },
            _ => {
                warn!("vcvars exited with failure");

                return Err(BuildError::BuildFailed(format!("{} {:?}", &self.platform.name(), exit)));
            }
        }

        Ok(())
    }
}