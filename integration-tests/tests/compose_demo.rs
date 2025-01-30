#![allow(dead_code)]

use integration_tests::test_components::{
    building::{BuildingModel, Thermostat},
    hourly_weather::HourlyWeather,
};
use twine_core::compose;

struct Demo;

struct DemoInput {
    occupancy: u32,
    temp_setpoint: f64,
    time: f64,
}

#[compose]
impl Demo {
    type Input = DemoInput;

    fn components() {
        let weather = HourlyWeather { time: input.time };

        let first_house = BuildingModel {
            occupancy: input.occupancy,
            outdoor_temp: weather.temperature,
            wind_speed: weather.wind_speed,
            thermostat: Thermostat {
                setpoint: input.temp_setpoint,
                auto: true,
            },
        };

        let second_house = BuildingModel {
            occupancy: input.occupancy,
            outdoor_temp: first_house.indoor_temp,
            wind_speed: 0.0,
            thermostat: Thermostat {
                setpoint: 20.0,
                auto: false,
            },
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Serializes and prints the given struct in JSON, TOML, and YAML formats.
    fn print_serialized<T: serde::Serialize + std::fmt::Debug>(label: &str, value: &T) {
        println!("\n==================== {label} ====================");
        println!("{value:#?}");

        println!("---------------------- JSON ----------------------");
        println!("{}", serde_json::to_string_pretty(value).unwrap());

        println!("---------------------- TOML ----------------------");
        println!("{}", toml::to_string(value).unwrap());

        println!("---------------------- YAML ----------------------");
        println!("{}", serde_yaml::to_string(value).unwrap());

        println!("=================================================\n");
    }

    #[test]
    fn inspect_demo_component() {
        let demo_config = DemoConfig::default();
        let demo_output = DemoOutput::default();

        print_serialized("Config", &demo_config);
        print_serialized("Output", &demo_output);

        assert!(
            serde_json::to_string(&demo_config).is_ok(),
            "Config JSON serialization failed."
        );
        assert!(
            serde_json::to_string(&demo_output).is_ok(),
            "Output JSON serialization failed."
        );
    }
}
