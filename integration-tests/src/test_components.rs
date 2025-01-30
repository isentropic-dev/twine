pub mod building {
    use serde::{Deserialize, Serialize};
    use twine_core::Component;

    /// A mock model of a building, used for integration tests.
    pub struct BuildingModel;

    impl Component for BuildingModel {
        type Config = Config;

        type Input = Input;

        type Output = Output;

        fn create(_config: Self::Config) -> impl Fn(Self::Input) -> Self::Output {
            |_input| Output::default()
        }
    }

    /// Configuration settings for the building model.
    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct Config {
        pub geometry: Geometry,
    }

    /// Represents the building’s input conditions.
    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct Input {
        pub outdoor_temp: f64,
        pub wind_speed: f64,
        pub occupancy: u32,
        pub thermostat: Thermostat,
    }

    /// Represents the building’s output results.
    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct Output {
        pub indoor_temp: f64,
        pub cooling_load: f64,
        pub heating_load: f64,
    }

    /// The physical dimensions of the building.
    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct Geometry {
        pub length: f64,
        pub width: f64,
        pub height: f64,
    }

    /// Thermostat settings for temperature control.
    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct Thermostat {
        pub setpoint: f64,
        pub auto: bool,
    }
}

pub mod hourly_weather {
    use serde::{Deserialize, Serialize};
    use twine_core::Component;

    /// A mock hourly weather provider, used for integration tests.
    pub struct HourlyWeather;

    impl Component for HourlyWeather {
        type Config = Config;

        type Input = Input;

        type Output = Output;

        fn create(_config: Self::Config) -> impl Fn(Self::Input) -> Self::Output {
            |_input| Output::default()
        }
    }

    /// Configuration settings for the weather provider.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Config {
        /// Path to the weather file containing hourly data.
        pub file_path: String,

        /// Factor applied to adjust weather data.
        pub adjustment_factor: f64,

        /// Temperature threshold for cold conditions.
        pub is_cold_criteria: f64,

        /// Number of rows to skip in the weather file.
        pub skip_rows: usize,

        /// Strategy for interpolating between hourly data points.
        pub interpolate: Interpolate,
    }

    /// Represents the weather provider’s input request.
    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct Input {
        /// The time for which weather data is requested.
        pub time: f64,
    }

    /// Represents the weather data output.
    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct Output {
        pub temperature: f64,
        pub pressure: f64,
        pub relative_humidity: f64,
        pub wind_speed: f64,
        pub is_cold: bool,
    }

    /// Interpolation strategy for estimating values between data points.
    #[derive(Debug, Default, Serialize, Deserialize)]
    pub enum Interpolate {
        /// Uses the nearest available data point.
        Nearest,

        /// Performs linear interpolation (default).
        #[default]
        Linear,

        /// Uses cubic interpolation.
        Cubic,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                file_path: "path/to/weather/file".to_string(),
                adjustment_factor: 1.0,
                is_cold_criteria: -40.0,
                skip_rows: 0,
                interpolate: Interpolate::default(),
            }
        }
    }
}
