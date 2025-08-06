fn main() {
    embed_resource::compile_for("res/batl.rc", ["batl"], embed_resource::NONE).manifest_required().unwrap();
}
