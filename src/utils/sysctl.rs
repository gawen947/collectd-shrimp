use sysctl::Sysctl;

pub fn get_string(key: &str) -> Result<String, sysctl::SysctlError> {
    let ctl = sysctl::Ctl::new(key)?;
    ctl.value_string()
}

pub fn get(key: &str) -> Result<sysctl::CtlValue, sysctl::SysctlError> {
    let ctl = sysctl::Ctl::new(key)?;
    ctl.value()
}
