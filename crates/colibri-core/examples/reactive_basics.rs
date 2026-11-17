use colibri_core::{batch, use_effect, use_memo, use_state, Scope};

fn main() {
    let count = use_state(1_i32);
    let doubled = {
        let count = count.clone();
        use_memo(move || count.get() * 2)
    };
    let mut scope = Scope::new();
    let effect = {
        let doubled = doubled.clone();
        use_effect(move || println!("double: {}", doubled.get()))
    };
    scope.track_effect(effect);
    batch(|| {
        count.set(2);
        count.set(3);
    });
    assert_eq!(doubled.get(), 6);
    drop(scope);
    count.set(4);
}
