fn main() {
	embed_resource::compile_for("res/yact.rc", ["yact"], embed_resource::NONE)
		.manifest_required()
		.unwrap();
}
