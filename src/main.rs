extern crate getopts;
extern crate consul;

use getopts::{optopt,reqopt,optflag,getopts,short_usage, usage};
use std::os;
use std::collections::HashMap;

use consul::{catalog, agent, health, structs};

fn list_services(addr: &str, tags: &str) {
    let mut all_services: HashMap<String, Vec<String>> = catalog::Catalog::new(addr).services(); 
    let local_services: HashMap<String, structs::Service> = agent::Agent::new(addr).services();
    for (_, service) in local_services.iter(){
        all_services.remove(service.Service);
    }
    let health_services: HashMap<&String, Vec<health::HealthService>> = all_services.iter()
        .map(|(k, _)| (k, health::Health::new(addr).service(k.as_slice(), tags)))
        .filter(|&(_ ,ref v)| v.len() > 0)
        .collect();
    println!("health services: {}", health_services)

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
        optopt("i", "input", "template of haproxy configuration file", "inputfile"),
        optopt("o", "output", "path of output haproxy configuration file, default: /etc/haproxy/haproxy.cfg", "outfile"),
        optopt("t", "tags", "tags that these services will filer, default: release", "tags"),
        optopt("a", "address", "http address of a consul agent, default: http://localhost:8500/v1", "address"),
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

    let address  = match matches.opt_str("a"){
        Some(m) => m,
        None => "http://localhost:8500/v1".to_string(),
    };
    let tags  = match matches.opt_str("t"){
        Some(m) => m,
        None => "release".to_string(),
    };
    list_services(address.as_slice(), tags.as_slice());
}
