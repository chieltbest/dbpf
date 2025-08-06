fn main() {
    embed_resource::compile_for("res/yape.rc", ["yape"], embed_resource::NONE).manifest_required().unwrap();
}
