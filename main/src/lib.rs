use stacker::remaining_stack;

pub fn get_remaining_stack() -> String {
    remaining_stack()
        .map(|x| format!("remaining stack: {x}"))
        .unwrap_or_else(|| "failed to get remaining stack".to_string())
}
