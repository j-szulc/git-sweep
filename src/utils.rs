pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn split_results<S, E>(vec_of_results: Vec<std::result::Result<S, E>>) -> (Vec<S>, Vec<E>) {
    let mut successes = Vec::new();
    let mut errors = Vec::new();
    for result in vec_of_results {
        match result {
            Ok(success) => successes.push(success),
            Err(error) => errors.push(error),
        }
    }
    (successes, errors)
}

pub fn 