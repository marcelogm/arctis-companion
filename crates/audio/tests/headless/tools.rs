use std::process::Command;

use serde_json::Value;

fn run(program: &str, args: &[&str]) -> String {
    let output = Command::new(program)
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("{program}: {error}"));
    assert!(output.status.success(), "{program} {args:?} failed");
    String::from_utf8(output.stdout).expect("utf-8 output")
}

fn dump() -> Vec<Value> {
    serde_json::from_str(&run("pw-dump", &[])).expect("pw-dump JSON")
}

fn props(object: &Value) -> &Value {
    &object["info"]["props"]
}

pub fn node_id(name: &str) -> Option<u64> {
    dump()
        .iter()
        .find(|object| props(object)["node.name"] == name)
        .and_then(|object| object["id"].as_u64())
}

pub fn configured_default() -> Option<String> {
    dump()
        .iter()
        .find(|object| object["props"]["metadata.name"] == "default")?["metadata"]
        .as_array()?
        .iter()
        .find(|entry| entry["key"] == "default.configured.audio.sink")?["value"]["name"]
        .as_str()
        .map(str::to_owned)
}

pub fn set_configured_default(name: &str) {
    let value = format!("{{ \"name\": \"{name}\" }}");
    run(
        "pw-metadata",
        &[
            "-n",
            "default",
            "0",
            "default.configured.audio.sink",
            &value,
            "Spa:String:JSON",
        ],
    );
}

pub fn set_props(name: &str, pod: &str) {
    let id = node_id(name).expect("node to set").to_string();
    run("pw-cli", &["set-param", &id, "Props", pod]);
}

pub fn volume_of(name: &str) -> Option<(Vec<f64>, bool)> {
    let object = dump()
        .into_iter()
        .find(|object| props(object)["node.name"] == name)?;
    let params = object["info"]["params"]["Props"].as_array()?;
    let volumes = params
        .iter()
        .find(|param| param.get("channelVolumes").is_some())?;
    let channels = volumes["channelVolumes"]
        .as_array()?
        .iter()
        .filter_map(Value::as_f64)
        .collect();
    Some((channels, volumes["mute"].as_bool()?))
}

pub fn manager_clients_of(pid: u32) -> usize {
    dump()
        .iter()
        .filter(|object| object["type"] == "PipeWire:Interface:Client")
        .map(props)
        .filter(|props| props["pipewire.sec.pid"].as_u64() == Some(u64::from(pid)))
        .filter(|props| props["media.category"] == "Manager")
        .count()
}
