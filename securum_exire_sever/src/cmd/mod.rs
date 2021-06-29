use std::ops::Add;

pub fn check_env() -> String {
    let home_path = std::env::var("HOME");
    if let Ok(home_path) = home_path {
        let conf_dir = home_path.add("/.securum_exire");
        let conf_dir_path = std::path::Path::new(&conf_dir);
        let conf_dir_is_present = conf_dir_path.is_dir();
        let config_file_loc = conf_dir.clone().add("/securum.toml.example");
        let config_file_path = std::path::Path::new(&config_file_loc);

        if conf_dir_is_present {
            let is_config_file_present = config_file_path.is_file();
            if is_config_file_present {
                return config_file_loc;
            } else {
                colour::e_red_ln!("error: config file not found [{}]", config_file_loc);
                colour::e_yellow_ln!("info: creating securum.toml.example");
                let res =std::fs::File::create(config_file_path);
                if res.is_err() {
                    colour::e_red_ln!("error: couldn't create {}", config_file_loc);
                    colour::e_yellow_ln!("please create the config file and try again!");
                    std::process::exit(1);
                } else {
                    colour::e_green_ln!("success: created the config file at {}, please refer to our documentation on more info about config file", config_file_loc);
                    return config_file_loc;
                }
            }
        } else {
            let res =std::fs::create_dir(&conf_dir);
            if res.is_err() {
                colour::e_red_ln!("error: couldn't create dir: {}", conf_dir);
                colour::e_yellow_ln!("please create the config file and the directory and try again!");
                std::process::exit(1);
            }
            let res =std::fs::File::create(&config_file_path);
            if res.is_err() {
                colour::e_red_ln!("error: couldn't create: {}", config_file_loc);
                colour::e_yellow_ln!("please create the config file and try again!");
                std::process::exit(1);
            } else {
                colour::e_green_ln!("success: created the config file at {}, please refer to our documentation on more info about config file", config_file_loc);
                return config_file_loc;
            }
        }
    } else {
        colour::e_red_ln!("error: HOME variable not found [specify HOME variable while running the program]");
        std::process::exit(1);
    }
}