#![allow(dead_code)]

// use integration_tests::test_components::{building::BuildingModel, hourly_weather::HourlyWeather};
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
            thermostat: building::Thermostat {
                setpoint: input.sure,
                auto: true,
            },
        };

        let second_house = BuildingModel {
            occupancy: input.occupancy,
            outdoor_temp: first_house.indoor_temp,
            wind_speed: 0.0,
            thermostat: building::Thermostat {
                setpoint: 20.0,
                auto: false,
            },
        };
    }
}
