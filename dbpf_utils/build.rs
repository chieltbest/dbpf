fn main() {
    embed_resource::compile_for("res/batl.rc", &["batl"], embed_resource::NONE).manifest_required().unwrap();
    embed_resource::compile_for("res/yact.rc", &["yact"], embed_resource::NONE).manifest_required().unwrap();
    embed_resource::compile_for("res/yape.rc", &["yape"], embed_resource::NONE).manifest_required().unwrap();
}
