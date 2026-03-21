use real_dep::Something;
use false_positive::FalsePositive;

fn main() {
    Something::new();
    FalsePositive::new();
}
