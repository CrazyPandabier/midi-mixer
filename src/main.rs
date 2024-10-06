use pulsectl::controllers::AppControl;
use pulsectl::controllers::DeviceControl;
use pulsectl::controllers::SinkController;

fn main() {
    // create handler that calls functions on playback devices and apps
    let mut handler = SinkController::create().unwrap();

    let devices = handler
        .list_devices()
        .expect("Could not get list of playback devices.");

    let applications = handler
        .list_applications()
        .expect("Could not get list of applications");

    println!("Playback Devices: ");
    for dev in devices.clone() {
        println!(
            "[{}] {}, Volume: {}",
            dev.index,
            dev.description.as_ref().unwrap(),
            dev.volume.print()
        );
    }

    for app in applications.clone() {
        println!(
            "[{}] {}, Volume: {}",
            app.index,
            {
                match app.proplist.get("application.process.binary") {
                    Some(binary_name) => std::str::from_utf8(binary_name).unwrap(),
                    None => "",
                }
            },
            app.volume.print()
        )
    }
}
