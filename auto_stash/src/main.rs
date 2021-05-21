fn main() {
    println!("Hello, world!");
    let notifier = filewatch::init_inotify();
    filewatch::start_watching("/home/benni/Coding/C", notifier);
}
