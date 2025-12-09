# 🪐 Planet Cargonauts

Welcome to **Cargonauts**!
We are a proud **TYPE C** planet in the Galaxy.

We aren't here to gather dust or act as a storage unit. We are here to transform the universe, one combination at a time. You bring the resources; we have the technology.

Project for the Advanced Programming Course 2025/2026.

## 🚀 Who We Are (Type C Specs)

As a Type C planet, we operate under strict constraints that require superior AI:

### 1. ⚡ "Just-in-Time" Energy Management
Unlike those lazy Type A planets that hoard batteries, we operate with a **Single EnergyCell**.
* This means we cannot waste a single sunray. Charge, use, repeat. Maximum efficiency is required.

### 2. ⚗️ The Master Crafters (Our Specialty)
Our resource combination rule is **UNBOUNDED**.
* Bring us any two resources supported by our recipes, and we will combine them. No limits. We are the factory of the galaxy.

### 3. ⛏️ Limited Extraction
Our ability to generate basic resources is limited (**At most one**).
* We are not a mine. Once we generate our specific resource (or hit the limit), the tap runs dry. Explorers come to us for crafting, not for raw materials.

### 4. 🛡️ Planetary Defense
We can build and host **at most one Rocket**.
* If an asteroid approaches and our only rocket isn't ready (or our single battery is dead), we are toast. The AI's priority is survival.

## 📡 Communication & Protocols

We use standard Rust `crossbeam_channel` channels to interact with the universe:
* **Orchestrator:** Sends us `Sunray` (for our single battery) and `Asteroid` (to test our nerves).
* **Explorers:** Our valued customers. They visit to combine precious resources.

## 🛠 Usage Instructions

To test the Cargonauts planet:

```bash
# Ensure you have the protocol dependencies and tests
cargo build
cargo test
```

To use the code: 

1. Add the dependency to the `Cargo.toml` file.
2. Use the `create_planet(...)` method `cargonauts::planet_ai::create_planet(...)`
