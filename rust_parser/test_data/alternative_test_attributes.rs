#[cfg(test)]
mod tests {
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

    #[custom_framework::async_test::test]
    async fn custom_multi_segment_test() {
        use custom_dep;
    }

    // Test attributes with arguments (runtime configuration, not test framework markers)
    #[tokio::test(flavor = "multi_thread")]
    async fn tokio_test_with_flavor() {
        use tokio_flavor_dep;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn tokio_test_multiple_args() {
        use tokio_multi_args_dep;
    }

    #[async_std::test(timeout = 1000)]
    async fn async_std_with_timeout() {
        use async_std_timeout_dep;
    }
}
