use notify_rust::Notification;

fn main() {
    Notification::new()
        .summary("Notification Test")
        .body("This is notification body.")
        .show()
        .unwrap();
}
