mod grandchild;

pub use grandchild::nested_greet;

pub fn greet() -> &'static str {
    "Hello from child module"
}
