pub fn not_found () -> String {
    "HTTP/1.1 404\n\n<h1>Page not found!</h1>".to_string()
}

#[cfg(test)]
mod tests {
    use super::not_found;

    #[test]
    fn not_found_response_is_correct() {
        let response = "HTTP/1.1 404\n\n<h1>Page not found!</h1>".to_string();
        assert_eq!(response, not_found());
    }
}
