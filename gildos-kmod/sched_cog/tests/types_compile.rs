use gildos_sched_cog::{set_class, Class};

#[test]
fn classes_are_distinct() {
    assert_ne!(Class::Rt, Class::Int);
    assert_ne!(Class::Int, Class::Bg);
    assert_ne!(Class::Bg, Class::Opt);
}

#[test]
fn stub_set_class_succeeds() {
    set_class(0, Class::Int, 0).expect("stub returns Ok");
}
