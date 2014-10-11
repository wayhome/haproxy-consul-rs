extern crate getopts;
extern crate serialize;

extern crate consul;
extern crate rustache;

use getopts::{optopt,reqopt,optflag,getopts,short_usage, usage};
use std::os;
use std::collections::HashMap;
use std::io::timer;
use std::time::Duration;
use std::io::MemWriter;
use serialize::json;


use consul::{catalog, agent, health, structs};

type Service = HashMap<String, Vec<health::HealthService>>;


fn list_extern_services(addr: &str, tags: &str) -> Service {
    let mut all_services: HashMap<String, Vec<String>> = catalog::Catalog::new(addr).services(); 
    let local_services: HashMap<String, structs::Service> = agent::Agent::new(addr).services();
    for (_, service) in local_services.iter(){
        all_services.remove(&service.Service);
    }
    let mut health_services = HashMap::new();
    for (k, _) in all_services.move_iter() {
        let v = health::Health::new(addr).service(k.as_slice(), tags);
        if v.len() > 0 {
            health_services.insert(k, v);
        }
    }
    health_services

}

fn main() {
    let args: Vec<String> = os::args();

    let program = args[0].clone();

    let help_text: &'static str = "
    Watch  services change in Consul and dynamically configures
    HAProxy backends. The process runs continuously, monitoring
    all the backends for changes. When there is a change, the template
    file is rendered to a destination path, and a reload command is
    invoked. This allows HAProxy configuration to be updated in real
    time using Consul.
        ";

    let opts = [
        optflag("h", "help", "print this help menu"),
        optopt("i", "input", "template of haproxy configuration file, default: /etc/hasu/haproxy.mustache", "inputfile"),
        optopt("o", "output", "path of output haproxy configuration file, default: /etc/haproxy/haproxy.cfg", "outfile"),
        optopt("", "tags", "tags that these services will filer, default: release", "tags"),
        optopt("", "address", "http address of a consul agent, default: http://localhost:8500/v1", "address"),
        optopt("", "interval", "check interval from consul, default:10", "interval"),
    ];
    let matches = match getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(_) => { println!("{}", short_usage(program.as_slice(), opts)); return }
    };
    if matches.opt_present("h") {
        println!("{}", short_usage(program.as_slice(), opts));
        println!("{}", usage(help_text, opts));
        return;
    }
    let template = match matches.opt_str("i") {
        Some(m) => {m}
        None => "/etc/hasu/haproxy.mustache".to_string()
    };

    let address  = match matches.opt_str("address"){
        Some(m) => m,
        None => "http://localhost:8500/v1".to_string(),
    };
    let tags  = match matches.opt_default("tags", "release"){
        Some(m) => m,
        None => "release".to_string(),
    };

    let interval: i64 = match matches.opt_default("interval", "10"){
        Some(m) => from_str(m.as_slice()).unwrap() ,
        None => 5,
    };

    println!("interval: {}", interval);
    loop {
        let extern_services = list_extern_services(address.as_slice(), tags.as_slice());
        let mut writer = MemWriter::new();
        rustache::render_file_from_json_string(template.as_slice(), json::encode(&extern_services).as_slice(), &mut writer);
        let result = String::from_utf8(writer.unwrap()).unwrap();
        println!("extern services: {}", extern_services);
        println!("{}", result);
        timer::sleep(Duration::seconds(interval));
    }
}
