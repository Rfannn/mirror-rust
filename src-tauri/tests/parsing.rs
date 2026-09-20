use mirror_rust_lib::devices::{model_from_info, parse_devices};

#[test]
fn parses_a_usb_device() {
    let output = "List of devices attached\nR58M12ABCDE\tdevice product:beyond1lte model:SM_G973F device:beyond1 transport_id:1\n";
    let devices = parse_devices(output);
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].serial, "R58M12ABCDE");
    assert_eq!(devices[0].state, "device");
    assert_eq!(devices[0].model, "SM G973F");
    assert_eq!(devices[0].port, "", "USB devices have no port");
}

#[test]
fn splits_a_wireless_address_into_ip_and_port() {
    let output =
        "List of devices attached\n192.168.1.42:5555\tdevice model:Pixel_8 transport_id:3\n";
    let devices = parse_devices(output);
    assert_eq!(devices[0].ip, "192.168.1.42");
    assert_eq!(devices[0].port, "5555");
    assert_eq!(devices[0].model, "Pixel 8");
}

#[test]
fn keeps_unauthorised_devices_visible() {
    let output = "List of devices attached\nABCD1234\tunauthorized usb:1-1 transport_id:4\n";
    let devices = parse_devices(output);
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].state, "unauthorized");
}

#[test]
fn ignores_the_header_and_daemon_notices() {
    let output = "* daemon not running; starting now at tcp:5037\n* daemon started successfully\nList of devices attached\n\n";
    assert!(parse_devices(output).is_empty());
}

#[test]
fn handles_an_empty_device_list() {
    assert!(parse_devices("List of devices attached\n\n").is_empty());
}

#[test]
fn extracts_the_model_from_an_info_string() {
    assert_eq!(
        model_from_info("product:x model:SM_M325F transport_id:2"),
        "SM M325F"
    );
    assert_eq!(model_from_info("product:x transport_id:2"), "");
}
