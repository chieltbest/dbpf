use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    embed_resource::compile("res/yact.rc");
    Ok(())
}
