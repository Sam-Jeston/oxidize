pub fn not_found () -> String {
    "<h1>Page not found!</h1>".to_string()
}

#[cfg(test)]
mod tests {
    use super::not_found;

    #[test]
    fn not_found_response_is_correct() {
        let response = "<h1>Page not found!</h1>".to_string();
        assert_eq!(response, not_found());
    }
}
