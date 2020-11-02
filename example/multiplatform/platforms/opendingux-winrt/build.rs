winrt::build!(
    dependencies
        os
    types
        windows::foundation::*
        windows::management::deployment::*
);

fn main() {
    build();
}
