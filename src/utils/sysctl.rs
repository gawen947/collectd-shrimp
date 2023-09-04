use sysctl::Sysctl;

pub fn get(key: &str) -> Result<String, sysctl::SysctlError> {
    let ctl = sysctl::Ctl::new(key)?;
    ctl.value_string()
}
