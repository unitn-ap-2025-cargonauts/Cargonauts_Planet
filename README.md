# 🪐 Planet Cargonauts

Welcome to **Cargonauts**!
We are a proud **TYPE C** planet in the Galaxy.

We aren't here to gather dust or act as a storage unit. We are here to transform the universe, one combination at a time. You bring the resources; we have the technology.

Project for the Advanced Programming Course 2025/2026.

## 🌍 Planet Properties
Our planet have the following properties:
- Basic resource: Carbon
- Complex resources: Supports all WG-defined complex resources
- Energy cells: 1
- Rockets: Supports 1 rocket (armed planet)

## 💡 Planet Logic

### Sunray handling
When receiving a sunray, the planet recharges its single energy cell.

### Asteroid handling
If no rocket is available, the planet constructs one (consuming the charged energy cell).
Once a rocket is available, it uses it to deflect the asteroid.

### Resource operations handling
At the moment, the planet gives priority to Explorer requests
(resource generation and resource combination) over its internal actions.

## 🛠 Usage Instructions

To test the Cargonauts planet:

```bash
# Ensure you have the protocol dependencies and tests
cargo build
cargo test
```
To add us to your docs:

```bash
cargo doc -p cargonauts --document-private-items

# Than, you can update the docs of your project
cargo doc --no-deps --open
```

To use the code: 

1. Add the dependency to the `Cargo.toml` file.
2. Use the `create_planet(...)` method `cargonauts::planet_ai::create_planet(...)`

## Contact us

You can find us in our [Telegram Group](https://t.me/+h6aCUupT45g1MmU8)
