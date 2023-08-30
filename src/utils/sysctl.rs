use sysctl::Sysctl;

pub fn get_string(key: &str) -> Result<String, sysctl::SysctlError> {
    let ctl = sysctl::Ctl::new(key)?;
    ctl.value_string()
}
