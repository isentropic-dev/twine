use integration_tests::test_components::{building, hourly_weather};
use twine_core::compose;

compose!(demo, {
    Input {
        time: f64,
        indoor: {
            occupancy: u32,
            pressure: f64,
            temp_setpoint: f64,
        },
        thermostat_control: {
            is_auto: bool,
        },
    }

    weather => hourly_weather {
        time,
    }

    first_house => building {
        occupancy: indoor.occupancy,
        outdoor_temp: weather.temperature,
        wind_speed: weather.wind_speed,
        thermostat: building::Thermostat {
            setpoint: indoor.temp_setpoint,
            auto: thermostat_control.is_auto,
        },
    }

    second_house => building {
        occupancy: indoor.occupancy,
        outdoor_temp: first_house.indoor_temp,
        wind_speed: 0.0,
        thermostat: building::Thermostat {
            setpoint: 20.0,
            auto: false,
        },
    }
});

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
        let demo_config = demo::Config::default();
        let demo_input = demo::Input::default();
        let demo_output = demo::Output::default();

        print_serialized("Config", &demo_config);
        print_serialized("Input", &demo_input);
        print_serialized("Output", &demo_output);

        assert!(
            serde_json::to_string(&demo_config).is_ok(),
            "Config JSON serialization failed."
        );
        assert!(
            serde_json::to_string(&demo_input).is_ok(),
            "Input JSON serialization failed."
        );
        assert!(
            serde_json::to_string(&demo_output).is_ok(),
            "Output JSON serialization failed."
        );
    }

    #[test]
    fn call_demo_component() {
        let demo_fn = demo::create(demo::Config::default());
        let output = demo_fn(demo::Input::default());
        println!("{output:#?}");
    }
}
