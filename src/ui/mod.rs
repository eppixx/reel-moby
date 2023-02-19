mod no_yaml_found;
mod yaml_found;

use anyhow::Result;

use crate::widget::service_switcher;
use crate::Opt;

pub fn create_ui(opt: &Opt) -> Result<()> {
    let service_result = service_switcher::ServiceSwitcher::new(&opt.file);
    match service_result {
        Some(switcher) => yaml_found::Ui::run(opt, switcher),
        _ => no_yaml_found::Ui::run(opt),
    }?;

    Ok(())
}
