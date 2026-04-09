use crate::ua::random_user_agent;

#[test]
fn random_agent_is_not_always_same() {
    let a1 = random_user_agent();
    let a2 = random_user_agent();
    assert_ne!(a1, a2);
}
