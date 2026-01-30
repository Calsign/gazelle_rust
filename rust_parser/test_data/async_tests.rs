#[cfg(test)] 
mod tests {
    use async_std;
    use actix_rt;
    use tokio;
    #[tokio::test]
    async fn tokio_test_function() {
        use tokio_dep;
    }

    #[async_std::test]
    async fn async_std_test_function() {
        use async_std_dep;
    }

    #[actix_rt::test]
    async fn actix_test_function() {
        use actix_dep;
    }

    #[test]
    fn regular_test() {
        use regular_dep;
    }
}
