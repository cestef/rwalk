use crate::{_main, cli::interactive::State};
use color_eyre::eyre::Result;
use log::error;

pub async fn run(state: &mut State) -> Result<()> {
    let res = _main(state.opts.clone()).await;
    match res {
        Ok(r) => {
            if let Some(root) = r.root {
                state.last_result = Some(root.lock().clone());
            }
        }
        Err(e) => {
            error!("{}", e);
            return Ok(());
        }
    }
    Ok(())
}
