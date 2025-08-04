# Twine

## What is Twine?

Twine is an open-source Rust framework for functional and composable system modeling.
It enables engineers and researchers to build complex system models from simple, reusable building blocks.
By emphasizing functional purity and strong typing, Twine makes system models more reliable, testable, and easier to understand.

## A Functional Approach to System Modeling

Twine models systems as compositions of functions with strongly-typed inputs and outputs that are deterministic, always returning the same output for the same input.
This functional approach allows users to build and understand larger systems from simple, reusable parts by ensuring:

- **Testability:** Deterministic functions are easier to test in isolation since they have no hidden state or side effects.
- **Parallelism:** Independent functions can be executed concurrently without race conditions.
- **Composability:** Systems can be built hierarchically from simple, reusable parts.
- **Reliability:** Pure functions eliminate side effects and make systems more predictable.

## Models and Simulations

Functions can be combined into a `Model` that represents a complete system's behavior.
Twine provides built-in numerical integration to evolve `Model` over time through a `Simulation`, enabling seamless simulation of dynamic systems while preserving functional purity.

## Built-in Components

Twine includes a library of ready-to-use components for common system modeling tasks, including:
- **Controllers:** Thermostats, PID controllers, etc.
- **Schedules:** Weekly and daily schedules, time-based control, etc.  
- **Thermal components:** Storage tanks, heat exchangers, mixing valves, etc.

Twine also provides a consistent API for thermodynamic and fluid property modeling, including interfaces to libraries like REFPROP and CoolProp.
