//! # Cargonauts Planet AI Module
//!
//! This module contains the implementation of the `PlanetAI` trait for the
//! Cargonauts planet. It defines how the planet handle messages from the
//! Orchestrator and the Explorer.
//!
//! Each handler is defined as a standalone function to keep the logic modular and clean.

use common_game::components::planet::*;
use common_game::components::resource::{BasicResource, BasicResourceType, Combinator, ComplexResource, ComplexResourceRequest, Generator};
use common_game::components::rocket::Rocket;
use common_game::components::sunray::Sunray;
use common_game::protocols::messages::*;
//use log::info; //TODO log?

struct CargonautsPlanet;

impl PlanetAI for CargonautsPlanet  {
    fn handle_orchestrator_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: OrchestratorToPlanet,
    ) -> Option<PlanetToOrchestrator> {
        todo!()
    }

    fn handle_explorer_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: ExplorerToPlanet
    ) -> Option<PlanetToExplorer> {
        match msg {
            ExplorerToPlanet::SupportedResourceRequest { explorer_id } => {
                //info!("SupportedResourceRequest received from explorer[{}]", explorer_id);
                handle_supported_resource_request(generator)
            },
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id } => {
                //info!("SupportedCombinationRequest received from explorer[{}]", explorer_id);
                handle_supported_combination_request(combinator)
            },
            ExplorerToPlanet::GenerateResourceRequest { explorer_id, resource } => {
                //info!("GenerateResourceRequest received from explorer[{}]. Ask for generate {:?}", explorer_id, resource);
                handle_generate_resource_request(state, generator, resource)
            },
            ExplorerToPlanet::CombineResourceRequest { explorer_id, msg } => {
                //info!("CombineResourceRequest received from explorer[{}]. Ask for craft {:?}", explorer_id, msg);
                handle_combine_resource_request(state, combinator, msg)
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

    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
    ) -> Option<Rocket> {
        todo!()
    }

    fn start(&mut self, state: &PlanetState) {
        todo!()
    }

    fn stop(&mut self, state: &PlanetState) {
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
/// - `generator`: Reference to the planet's generator
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
    generator: &Generator,
) -> Option<PlanetToExplorer> {
    let resource_list = Some(generator.all_available_recipes());
    Some(PlanetToExplorer::SupportedResourceResponse { resource_list })
}

/// This handler returns a `SupportedCombinationResponse` message that wrap the list of complex resources
/// that the planet can currently generate
///
/// # Parameters
/// - `combinator`: Reference to the planet's combinator
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
    combinator: &Combinator,
) -> Option<PlanetToExplorer> {
    let combination_list = Some(combinator.all_available_recipes());
    Some(PlanetToExplorer::SupportedCombinationResponse  { combination_list })
}

/// This handler processes a request to generate a basic resource using the planet's generator, 
/// if energy is available.
/// It returns a `GenerateResourceResponse` message containing the generated resource.
///
/// # Parameters
/// - `state`: Mutable reference to the planet state.
/// - `generator`: Reference to the planet's generator.
/// - `req_resource`: The type of basic resource the explorer is requesting to generate.
///
/// # Returns
/// `Some(PlanetToExplorer::GenerateResourceResponse)` containing:
/// - `Some(BasicResource::Carbon)` on successful generation.
/// - `None` if the planet has no charged energy cell.
///
/// # Panics
/// - If the requested resource type is not supported by the generator.
/// - If the generator reports an unexpected error while crafting.
///
/// # Logic
/// - Retrieve the energy cell and check if it is charged
/// - If charged:
///     - Attempt to generate the requested basic resource via the generator
///     - Wrap the generated resource in a `GenerateResourceResponse` message and return it.
/// - Else:
///     - Wrap a `None` in a `GenerateResourceResponse` message and return it.
fn handle_generate_resource_request(
    state: &mut PlanetState,
    generator: &Generator,
    req_resource: BasicResourceType,
) -> Option<PlanetToExplorer> {
    let mut resource: Option<BasicResource> = None;
    let energy_cell = state.cell_mut(0);
    if energy_cell.is_charged(){
        match req_resource {
            BasicResourceType::Carbon => {
                match generator.make_carbon(energy_cell){
                    Ok( r) => resource = Some(BasicResource::Carbon(r)),
                    Err(e) => panic!("{:?}", e) //TODO right?
                }
            },
            _ => panic!("Unexpected resource type") //TODO right?
        }
    }
    Some(PlanetToExplorer::GenerateResourceResponse { resource })
}

/// This handler processes a request to combine two basic or complex resources
/// into a new complex resource using the planet's combinator, if energy is available.
/// It returns a `CombineResourceResponse` message containing the newly
/// crafted complex resource.
///
/// # Parameters
/// - `state`: Mutable reference to the planet state.
/// - `combinator`: Reference to the planet's combinator.
/// - `msg`: A `ComplexResourceRequest` specifying which complex resource the
///   explorer want to craft, with the ingredients required.
///
/// # Returns
/// `Some(PlanetToExplorer::CombineResourceResponse)` containing:
/// - `Some(ComplexResource::X)` if the combination succeeds.
/// - `None` if the planet has no charged energy cell.
///
/// # Panics
/// - If the requested complex resource type is not supported by the combinator.
/// - If the combinator reports an unexpected error while crafting.
///
/// # Logic
/// - Retrieve the energy cell and check if it is charged.
/// - If charged:
///     - Attempt to combine the provided ingredients using the combinator.
///     - Wrap the produced resource in a `CombineResourceResponse` message and return it.
/// - Else:
///     - Wrap a `None` in a `CombineResourceResponse` message and return it.
fn handle_combine_resource_request(
    state: &mut PlanetState,
    combinator: &Combinator,
    msg: ComplexResourceRequest,
) -> Option<PlanetToExplorer> {
    let mut complex_response: Option<ComplexResource> = None;
    let energy_cell = state.cell_mut(0);
    if energy_cell.is_charged(){
        match msg {
            ComplexResourceRequest::Diamond(carbon1, carbon2) => {
                match combinator.make_diamond(carbon1, carbon2, energy_cell){
                    Ok( r) => complex_response = Some(ComplexResource::Diamond(r)),
                    Err(e) => panic!("{:?}", e) //TODO right?
                }
            },
            ComplexResourceRequest::Life(water, carbon) => {
                match combinator.make_life(water, carbon, energy_cell){
                    Ok( r) => complex_response = Some(ComplexResource::Life(r)),
                    Err(e) => panic!("{:?}", e) //TODO right?
                }
            },
            _ => panic!("Unexpected resource type") //TODO right?
        }
    }
    Some(PlanetToExplorer::CombineResourceResponse { complex_response })
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

        let result = handle_supported_resource_request(planet.generator()); //TODO wait for the add of getter and than test

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

        let result = handle_supported_combination_request(planet.combinator()); //TODO wait for the add of getter and than test

        assert!(result.is_some());

        if let Some(PlanetToExplorer::SupportedCombinationResponse { combination_list }) = result {
            assert!(combination_list.is_some());
            let resource_vec = combination_list.unwrap();

            let result_set: HashSet<ComplexResourceType> = resource_vec.into_iter().collect();
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