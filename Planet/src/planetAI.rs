//! # Cargonauts Planet AI Module
//!
//! This module contains the implementation of the `PlanetAI` trait for the
//! Cargonauts planet. It defines how the planet handle messages from the
//! Orchestrator and the Explorer.
//!
//! Each handler is defined as a standalone function to keep the logic modular and clean.

use std::sync::Arc;
use common_game::components::planet::*;
use common_game::components::rocket::Rocket;
use common_game::components::sunray::Sunray;
use common_game::protocols::messages::*;

struct CargonautsPlanet;

impl PlanetAI for CargonautsPlanet  {
    fn handle_orchestrator_msg(&mut self, state: &mut PlanetState, msg: OrchestratorToPlanet) -> Option<PlanetToOrchestrator> {
        match msg {
            OrchestratorToPlanet::Sunray(ray) => {
                handle_sunray(state, ray)
            },
            OrchestratorToPlanet::Asteroid(_) => None, //Handled in start method
            //OrchestratorToPlanet::StartPlanetAI(_) => {}
            OrchestratorToPlanet::StopPlanetAI(_) => None, //Handled in start method
            //OrchestratorToPlanet::ManualStopPlanetAI(_) => {}
            //OrchestratorToPlanet::ManualStartPlanetAI(_) => {}
            OrchestratorToPlanet::InternalStateRequest(msg) => {
                handle_internal_state_request_orch(state, msg)
            }
            _ => None //TODO Remove after is defined where to manage StartPlanetAI, ManualStopPlanetAI, ManualStartPlanetAI
        }
    }

    fn handle_explorer_msg(&mut self, state: &mut PlanetState, msg: ExplorerToPlanet) -> Option<PlanetToExplorer> {
        match msg {
            ExplorerToPlanet::SupportedResourceRequest { explorer_id } => {
                handle_supported_resource_request(state, explorer_id)
            },
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id } => {
                handle_supported_combination_request(state, explorer_id)
            },
            ExplorerToPlanet::GenerateResourceRequest { explorer_id, msg } => {
                handle_generate_resource_request(state, explorer_id, msg)
            },
            ExplorerToPlanet::CombineResourceRequest { explorer_id, msg } => {
                handle_combine_resource_request(state, explorer_id, msg)
            },
            ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id } => {
                handle_energy_cell_request(state, explorer_id)
            },
            ExplorerToPlanet::InternalStateRequest { explorer_id } => {
                handle_internal_state_request(state, explorer_id)
            }
        }
    }

    fn handle_asteroid(&mut self, state: &mut PlanetState) -> Option<Rocket> {
        todo!()
    }

    fn start(&mut self, state: &PlanetState) {
        todo!()
    }

    fn stop(&mut self) {
        todo!()
    }
}

// === OrchestratorToPlanet Handler ================================================================

fn handle_sunray(
    state: &mut PlanetState,
    ray: Sunray,
) -> Option<PlanetToOrchestrator> {
    todo!()
}

fn handle_internal_state_request_orch(
    state: &mut PlanetState,
    msg: InternalStateRequestMsg,
) -> Option<PlanetToOrchestrator> {
    todo!()
}

// === ExplorerToPlanet Handler ====================================================================
/// This handler returns a `SupportedResourceResponse` message that wrap the list of basic resources
/// that the planet can currently generate
///
/// # Parameters
/// - `state`: Reference to the planet state
/// - `explorer_id`: ID of the requesting explorer
///
/// # Returns
/// `Some(PlanetToExplorer::SupportedResourceResponse)` on success.
///
/// # Panics
/// This function does not panic.
///
/// # Logic
/// The planet can craft basic resources, so the handler:
/// - Get the set of available basic resource from the planet generator
/// - Collect the set into a vec
/// - Wrap the vector in a `SupportedResourceResponse` message and return it
fn handle_supported_resource_request(
    state: &PlanetState,
    explorer_id: u32,
) -> Option<PlanetToExplorer> {
    let resource_list = Some(
        state.generator.all_available_recipes()
            .into_iter()
            .collect()
    );
    Some(PlanetToExplorer::SupportedResourceResponse { resource_list })
}

/// This handler returns a `SupportedCombinationResponse` message that wrap TODO
///
/// # Parameters
/// - `state`: Reference to the planet state
/// - `explorer_id`: ID of the requesting explorer
///
/// # Returns
/// `Some(PlanetToExplorer::SupportedCombinationResponse)` on success.
///
/// # Panics
/// This function does not panic. TODO
///
/// # Logic
/// TODO
fn handle_supported_combination_request(
    state: &PlanetState,
    explorer_id: u32,
) -> Option<PlanetToExplorer> {
    let combination_list = Some(Arc::new(state.combinator)); //TODO
    Some(PlanetToExplorer::SupportedCombinationResponse  { combination_list })
}

fn handle_generate_resource_request(
    state: &mut PlanetState,
    explorer_id: u32,
    msg: GenerateResourceRequest,
) -> Option<PlanetToExplorer> {
    todo!()
}

fn handle_combine_resource_request(
    state: &mut PlanetState,
    explorer_id: u32,
    msg: CombineResourceRequest,
) -> Option<PlanetToExplorer> {
    todo!()
}

fn handle_energy_cell_request(
    state: &mut PlanetState,
    explorer_id: u32,
) -> Option<PlanetToExplorer> {
    todo!()
}

fn handle_internal_state_request(
    state: &mut PlanetState,
    explorer_id: u32,
) -> Option<PlanetToExplorer> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::mpsc;
    use common_game::components::resource::{BasicResourceType, ComplexResourceType};

    // Function that create a Planet with specific arguments
    fn create_planet(
        id: u32,
        planet_type: PlanetType,
        gen_rules: Vec<BasicResourceType>,
        comb_rules: Vec<ComplexResourceType>
    ) -> Planet<CargonautsPlanet> {
        let (to_orchestrator_tx, _to_orchestrator_rx) = mpsc::channel(); // Planet -> Orchestrator
        let (_from_orchestrator_tx, from_orchestrator_rx) = mpsc::channel(); // Orchestrator -> Planet
        let (to_explorer_tx, _to_explorer_rx) = mpsc::channel(); // Planet -> Explorer
        let (_from_explorer_tx, from_explorer_rx) = mpsc::channel(); // Explorer -> Planet

        Planet::new(
            id,
            planet_type,
            CargonautsPlanet,
            gen_rules,
            comb_rules,
            (from_orchestrator_rx, to_orchestrator_tx),
            (from_explorer_rx, to_explorer_tx),
        ).expect("Failed to create planet")
    }

    #[test]
    fn test_base_handle_supported_resource_request() {
        let planet_id = 0;
        let planet_type = PlanetType::C;
        let gen_rules = vec![BasicResourceType::Carbon];
        let comb_rules = vec![];
        let planet = create_planet(planet_id, planet_type, gen_rules.clone(), comb_rules);
        let explorer_id = 1;

        let result = handle_supported_resource_request(planet.state(), explorer_id);

        assert!(result.is_some());

        if let Some(PlanetToExplorer::SupportedResourceResponse { resource_list }) = result {
            assert!(resource_list.is_some());
            let resource_vec = resource_list.unwrap();

            let result_set: HashSet<BasicResourceType> = resource_vec.into_iter().collect();
            let expected_set: HashSet<BasicResourceType> = gen_rules.into_iter().collect();
            assert_eq!(result_set, expected_set);
        } else {
            panic!("Expected SupportedResourceResponse variant");
        }
    }
}