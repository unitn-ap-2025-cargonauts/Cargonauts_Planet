//! # Cargonauts Planet AI Module
//!
//! This module contains the implementation of the `PlanetAI` trait for the
//! Cargonauts planet. It defines how the planet handle messages from the
//! Orchestrator and the Explorer.
//!
//! Each handler is defined as a standalone function to keep the logic modular and clean.

use common_game::components::planet::*;
use common_game::components::resource::{BasicResourceType, ComplexResourceRequest};
use common_game::components::rocket::Rocket;
use common_game::components::sunray::Sunray;
use common_game::protocols::messages::*;
//use log::info; //TODO log?

struct CargonautsPlanet;

impl PlanetAI for CargonautsPlanet  {
    fn handle_orchestrator_msg(&mut self, state: &mut PlanetState, msg: OrchestratorToPlanet) -> Option<PlanetToOrchestrator> {
        match msg {
            OrchestratorToPlanet::Sunray(ray) => {
                //info!("Sunray received from orchestrator");
                handle_sunray(state, ray)
            },
            OrchestratorToPlanet::Asteroid(_) => None, //Handled in start method
            //OrchestratorToPlanet::StartPlanetAI(_) => {}
            OrchestratorToPlanet::StopPlanetAI(_) => None, //Handled in start method
            //OrchestratorToPlanet::ManualStopPlanetAI(_) => {}
            //OrchestratorToPlanet::ManualStartPlanetAI(_) => {}
            OrchestratorToPlanet::InternalStateRequest(msg) => {
                //info!("InternalStateRequest received from orchestrator");
                handle_internal_state_request_orch(state, msg)
            }
            _ => None //TODO Remove after is defined where to manage StartPlanetAI, ManualStopPlanetAI, ManualStartPlanetAI
        }
    }

    fn handle_explorer_msg(&mut self, state: &mut PlanetState, msg: ExplorerToPlanet) -> Option<PlanetToExplorer> {
        match msg {
            ExplorerToPlanet::SupportedResourceRequest { explorer_id } => {
                //info!("SupportedResourceRequest received from explorer[{}]", explorer_id);
                handle_supported_resource_request(state)
            },
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id } => {
                //info!("SupportedCombinationRequest received from explorer[{}]", explorer_id);
                handle_supported_combination_request(state)
            },
            ExplorerToPlanet::GenerateResourceRequest { explorer_id, resource } => {
                //info!("GenerateResourceRequest received from explorer[{}]. Ask for generate {:?}", explorer_id, resource);
                handle_generate_resource_request(state, explorer_id, resource)
            },
            ExplorerToPlanet::CombineResourceRequest { explorer_id, msg } => {
                //info!("CombineResourceRequest received from explorer[{}]. Ask for craft {:?}", explorer_id, msg);
                handle_combine_resource_request(state, explorer_id, msg)
            },
            ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id } => {
                //info!("AvailableEnergyCellRequest received from explorer[{}]", explorer_id);
                handle_energy_cell_request(state)
            },
            ExplorerToPlanet::InternalStateRequest { explorer_id } => {
                //info!("InternalStateRequest received from explorer[{}]", explorer_id);
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
///
/// # Returns
/// `Some(PlanetToExplorer::SupportedResourceResponse)`
///
/// # Panics
/// This function does not panic.
///
/// # Logic
/// The planet can craft basic resources, so the handler:
/// - Get the set of available basic resource from the planet generator
/// - Wrap the set in a `SupportedResourceResponse` message and return it
fn handle_supported_resource_request(
    state: &PlanetState,
) -> Option<PlanetToExplorer> {
    let resource_list = Some(state.generator.all_available_recipes());
    Some(PlanetToExplorer::SupportedResourceResponse { resource_list })
}

/// This handler returns a `SupportedCombinationResponse` message that wrap the list of complex resources
/// that the planet can currently generate
///
/// # Parameters
/// - `state`: Reference to the planet state
///
/// # Returns
/// `Some(PlanetToExplorer::SupportedCombinationResponse)`
///
/// # Panics
/// This function does not panic.
///
/// # Logic
/// The planet can craft complex resources, so the handler:
/// - Get the set of available complex resource from the planet combinator
/// - Wrap the set in a `SupportedCombinationResponse` message and return it
fn handle_supported_combination_request(
    state: &PlanetState,
) -> Option<PlanetToExplorer> {
    let combination_list = Some(state.combinator.all_available_recipes());
    Some(PlanetToExplorer::SupportedCombinationResponse  { combination_list })
}

///TODO fn handle_generate_resource_request description
fn handle_generate_resource_request(
    state: &mut PlanetState,
    explorer_id: u32,
    req_resource: BasicResourceType,
) -> Option<PlanetToExplorer> {
    /*let mut resource: Option<BasicResource> = None;
    let energy_cell = state.cell_mut(0);
    if energy_cell.is_charged(){
        match req_resource {
            BasicResourceType::Carbon => {
                match state.generator.make_carbon(energy_cell){ //TODO
                    Ok( r) => resource = Some(BasicResource::Carbon(r)),
                    Err(e) => panic!("{}", e) //TODO propagate the error?
                }
            },
            _ => panic!("Unexpected resource type") //TODO use Err()?
        }
    }
    Some(PlanetToExplorer::GenerateResourceResponse { resource })*/
    todo!()
}

///TODO fn handle_combine_resource_request description
fn handle_combine_resource_request(
    state: &mut PlanetState,
    explorer_id: u32,
    msg: ComplexResourceRequest,
) -> Option<PlanetToExplorer> {
    todo!()
}

/// This handler returns an `AvailableEnergyCellResponse` message containing
/// the number of currently charged energy cells available on the planet.
/// Since the planet has only one energy cell, the value can only be 0 or 1.
///
/// # Parameters
/// - `state`: Reference to the planet state
///
/// # Returns
/// `Some(PlanetToExplorer::AvailableEnergyCellResponse)` with `available_cells` set to:
/// - **0**: if the energy cell is discharged
/// - **1**: if the energy cell is charged
///
/// # Panics
/// This function does not panic.
///
/// # Logic
/// The handler:
/// - Initializes a counter to 0
/// - Accesses the energy cell and increments the counter if it is charged
/// - Wraps the counter inside an `AvailableEnergyCellResponse` message and returns it
fn handle_energy_cell_request(
    state: &PlanetState,
) -> Option<PlanetToExplorer> {
    let mut available_cells = 0;
    if state.cell(0).is_charged() {
        available_cells += 1;
    }
    Some(PlanetToExplorer::AvailableEnergyCellResponse { available_cells })
}

///TODO fn handle_internal_state_request description
fn handle_internal_state_request(
    state: &PlanetState,
    explorer_id: u32,
) -> Option<PlanetToExplorer> {
    /*Some(PlanetToExplorer::InternalStateResponse { planet_state: state })*/ //TODO find out the utility of this msg and ask if is need to pass the only ref and not the whole ownership
    todo!()
}

// === Utilities Functions ================================================================


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::collections::HashSet;
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

        let result = handle_supported_resource_request(planet.state());

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

    #[test]
    fn test_base_handle_supported_combination_request() {
        let planet_id = 0;
        let planet_type = PlanetType::C;
        let gen_rules = vec![BasicResourceType::Carbon];
        let comb_rules = vec![ComplexResourceType::Diamond, ComplexResourceType::Life];
        let planet = create_planet(planet_id, planet_type, gen_rules, comb_rules.clone());

        let result = handle_supported_combination_request(planet.state());

        assert!(result.is_some());

        if let Some(PlanetToExplorer::SupportedCombinationResponse { combination_list }) = result {
            assert!(combination_list.is_some());
            let resource_vec = combination_list.unwrap();

            let result_set: HashSet<ComplexResourceType> = resource_vec.into_iter().collect(); //TODO wait for messages.rs fix by JM, than test
            let expected_set: HashSet<ComplexResourceType> = comb_rules.into_iter().collect();
            assert_eq!(result_set, expected_set);
        } else {
            panic!("Expected SupportedCombinationResponse variant");
        }
    }

    /*#[test]
    fn test_base_handle_energy_cell_request_charge() {
        todo!()
    }

    #[test]
    fn test_base_handle_energy_cell_request_discharge() {
        todo!()
    }*/
}