use duration_string::*;
use envconfig::Envconfig;
use std::time::Duration;

struct MyDuration(Duration);
impl std::ops::Deref for MyDuration {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for MyDuration {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Debug for MyDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for MyDuration {
    type Err = duration_string::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let d = DurationString::try_from(String::from(s))?.into();
        Ok(MyDuration(d))
    }
}

impl From<MyDuration> for Duration {
    fn from(val: MyDuration) -> Self {
        val.0
    }
}

#[derive(Envconfig)]
struct Config {
    #[envconfig(from = "DATABASE_URL")]
    database_url: String,

    #[envconfig(from = "PORT", default = "8080")]
    port: u16,

    #[envconfig(from = "DURATION", default = "10s")]
    duration: MyDuration,
}

fn main() {
    match Config::init_from_env() {
        Ok(config) => {
            println!("Database URL: {}", config.database_url);
            println!("Port: {}", config.port);
            println!("Duration: {:?}", config.duration);
        }
        Err(err) => {
            eprintln!("Error loading config: {}", err);
        }
    }
}
