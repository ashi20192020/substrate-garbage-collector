use crate::mock::*;
use frame_support::{
	assert_ok,
	traits::OnInitialize,
};
pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		System::set_block_number(System::block_number() + 1);
		Scheduler::on_initialize(System::block_number());
		GarbageCollector::schedule_cleanup(n - 1);
	}
}
#[test]
fn tree_cleanup_test() {
	new_test_ext().execute_with(|| {

		assert_ok!(GarbageCollector::increase_index(frame_system::RawOrigin::Root.into()));
		assert_ok!(Tree::water(frame_system::RawOrigin::Root.into()));
		let call = Box::new(Call::Tree(pallet_tree::Call::cleanup {}));
		assert_ok!(GarbageCollector::submit_pallet(
			frame_system::RawOrigin::Root.into(),
			call
		));
		assert_eq!(Tree::height(), 5);
		run_to_block(11);

		// Proofs that indeed clenup on tree pallet has triggered
		System::assert_has_event(Event::Tree(pallet_tree::Event::HeightDecreased(0)));
		let height = Tree::height();
		// Tree has initial height of 5, and due to cleanup it will be 0
		assert_eq!(height, 0);
	});
}
