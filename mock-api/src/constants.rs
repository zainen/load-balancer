use dotenvy::dotenv;
use lazy_static::lazy_static;

lazy_static!{
    pub static ref PORT: u32 = set_port();
}


fn set_port() -> u32 {
    dotenv().ok();


    let args = std::env::args().collect::<Vec<String>>();
    
    let port_arg = args[1].parse::<u32>();

    if port_arg.is_ok() {
        return port_arg.unwrap()
    }

    let env_port_str = std::env::var(env::PORT_ENV_VAR);

    env_port_str.clone().expect("PORT REQUIRED: ENV NOT SET").parse::<u32>().expect(&format!("Failed to parse port to u32: {}", env_port_str.unwrap()))
}


pub mod env {
    pub const PORT_ENV_VAR: &str = "PORT";
}
