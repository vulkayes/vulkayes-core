use crate::entry;

#[test]
fn enumerate_layers_and_extensions() {
	crate::test::setup_testing_logger();

	let entry = entry::Entry::new().expect("Could not create entry");

	entry.instance_layers().unwrap().for_each(|layer| {
		log::info!("Layer {}", layer);
	});

	entry.instance_extensions().unwrap().for_each(|extension| {
		log::info!("Extension {}", extension);
	});
}
