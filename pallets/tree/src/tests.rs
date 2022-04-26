use crate::mock::*;
use frame_support::assert_ok;

#[test]
fn tree_increase_decrease_height() {
	new_test_ext().execute_with(|| {

		System::set_block_number(1);
		assert_ok!(Tree::water(frame_system::RawOrigin::Root.into()));

		System::assert_last_event(Event::Tree(crate::Event::HeightIncreased(10)));

		System::set_block_number(4);
		// try to cleanup before last_watering_time + 5
		assert_ok!(Tree::cleanup(frame_system::RawOrigin::Root.into()));

		// no HeightDecreased event means  cleanup did not get executed.
		System::assert_last_event(Event::Tree(crate::Event::HeightIncreased(10)));

		System::set_block_number(6);
		assert_ok!(Tree::cleanup(frame_system::RawOrigin::Root.into()));

		System::assert_last_event(Event::Tree(crate::Event::HeightDecreased(5)));
	});
}
